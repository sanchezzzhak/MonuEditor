use anyhow::{Context, Result, anyhow};
use ash::{Entry, Instance, Device, khr, vk};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::ffi::{CStr, CString};
use winit::window::Window;

pub struct QueueFamilyIndices {
    pub graphics: u32,
    pub present: u32,
    /// Отдельная очередь для фоновой загрузки чанков ландшафта в видеопамять (если железо поддерживает)
    pub transfer: u32,
}

pub struct VulkanContext {
    pub entry: Entry,
    pub instance: Instance,
    pub surface_loader: khr::surface::Instance,
    pub surface: vk::SurfaceKHR,
    pub physical_device: vk::PhysicalDevice,
    pub device: Device,
    pub queue_indices: QueueFamilyIndices,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub transfer_queue: vk::Queue,
    pub swapchain_loader: khr::swapchain::Device,
}

impl VulkanContext {
    pub fn new(window: &Window) -> Result<Self> {
        // 1. Загружаем библиотеку vulkan-1.dll / libvulkan.so
        let entry = unsafe { Entry::load() }
            .context("Не удалось загрузить драйвер Vulkan. Убедитесь, что драйверы видеокарты обновлены.")?;

        // 2. Создаем Vulkan Instance
        let app_name = CString::new("MonuEngine")?;
        let engine_name = CString::new("MonuEngine Core")?;

        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            // Ставим 1.1 для максимальной совместимости со старым железом и встройками
            .api_version(vk::API_VERSION_1_1);

        let display_handle = window.display_handle()?.as_raw();
        let extension_names =
            ash_window::enumerate_required_extensions(display_handle)?.to_vec();

        // В debug-сборке включаем слои валидации, чтобы Vulkan ругался на любые ошибки памяти
        #[cfg(debug_assertions)]
        let layer_names = [CString::new("VK_LAYER_KHRONOS_validation")?];
        #[cfg(not(debug_assertions))]
        let layer_names: [CString; 0] = [];

        let layer_names_raw: Vec<*const i8> = layer_names.iter().map(|name| name.as_ptr()).collect();

        // Проверяем, установлены ли слои валидации на ПК (если нет — не падаем, просто отключаем)
        let available_layers = unsafe { entry.enumerate_instance_layer_properties()? };
        let has_validation = layer_names.iter().all(|req| {
            available_layers.iter().any(|avail| {
                let name = unsafe { CStr::from_ptr(avail.layer_name.as_ptr()) };
                name == req.as_c_str()
            })
        });

        let active_layers = if has_validation {
            layer_names_raw.as_slice()
        } else {
            &[]
        };

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(active_layers);

        let instance = unsafe { entry.create_instance(&create_info, None)? };

        // 3. Создаем Surface (связь окна ОС с Vulkan)
        let window_handle = window.window_handle()?.as_raw();
        let surface = unsafe {
            ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)?
        };
        let surface_loader = khr::surface::Instance::new(&entry, &instance);

        // 4. Выбираем лучшую доступную видеокарту (даже если это старая встройка)
        let (physical_device, queue_indices) =
            Self::pick_physical_device(&instance, &surface_loader, surface)?;

        // 5. Создаем логическое устройство и очереди
        let (device, graphics_queue, present_queue, transfer_queue) =
            Self::create_logical_device(&instance, physical_device, &queue_indices)?;

        let swapchain_loader = khr::swapchain::Device::new(&instance, &device);

        Ok(Self {
            entry,
            instance,
            surface_loader,
            surface,
            physical_device,
            device,
            queue_indices,
            graphics_queue,
            present_queue,
            transfer_queue,
            swapchain_loader,
        })
    }

    fn pick_physical_device(
        instance: &Instance,
        surface_loader: &khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> Result<(vk::PhysicalDevice, QueueFamilyIndices)> {
        let devices = unsafe { instance.enumerate_physical_devices()? };
        if devices.is_empty() {
            return Err(anyhow!("В системе не найдено видеокарт с поддержкой Vulkan"));
        }

        let mut best_device = None;
        let mut best_score = 0;

        for pdevice in devices {
            if let Some(indices) = Self::find_queue_families(instance, surface_loader, surface, pdevice) {
                let props = unsafe { instance.get_physical_device_properties(pdevice) };
                let name = unsafe { CStr::from_ptr(props.device_name.as_ptr()) }.to_string_lossy();

                // Оцениваем карту: дискретная получает +1000 баллов, встройка +100 (но тоже подойдет!)
                let mut score = match props.device_type {
                    vk::PhysicalDeviceType::DISCRETE_GPU => 1000,
                    vk::PhysicalDeviceType::INTEGRATED_GPU => 100,
                    _ => 10,
                };

                // Учитываем максимальный размер текстур как косвенный показатель мощности
                score += props.limits.max_image_dimension2_d / 1024;

                log::info!("Найдена GPU: {} (очки: {})", name, score);

                if score > best_score {
                    best_score = score;
                    best_device = Some((pdevice, indices, name.into_owned()));
                }
            }
        }

        let (device, indices, name) = best_device
            .ok_or_else(|| anyhow!("Ни одна видеокарта не поддерживает необходимые очереди Vulkan"))?;

        println!("Выбрана видеокарта: {}", name);
        Ok((device, indices))
    }

    fn find_queue_families(
        instance: &Instance,
        surface_loader: &khr::surface::Instance,
        surface: vk::SurfaceKHR,
        pdevice: vk::PhysicalDevice,
    ) -> Option<QueueFamilyIndices> {
        let queue_families = unsafe { instance.get_physical_device_queue_family_properties(pdevice) };

        let mut graphics = None;
        let mut present = None;
        let mut dedicated_transfer = None;

        for (i, family) in queue_families.iter().enumerate() {
            let idx = i as u32;

            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics = Some(idx);
            }

            let present_support = unsafe {
                surface_loader
                .get_physical_device_surface_support(pdevice, idx, surface)
                .unwrap_or(false)
            };
            if present_support {
                present = Some(idx);
            }

            // Ищем выделенную очередь только для копирования (без GRAPHICS), чтобы грузить чанки без фризов
            if family.queue_flags.contains(vk::QueueFlags::TRANSFER)
                && !family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                dedicated_transfer = Some(idx);
            }
        }

        let g = graphics?;
        let p = present?;
        // Если на старой карте нет отдельной Transfer-очереди, используем графическую
        let t = dedicated_transfer.unwrap_or(g);

        Some(QueueFamilyIndices {
            graphics: g,
            present: p,
            transfer: t,
        })
    }

    fn create_logical_device(
        instance: &Instance,
        pdevice: vk::PhysicalDevice,
        indices: &QueueFamilyIndices,
    ) -> Result<(Device, vk::Queue, vk::Queue, vk::Queue)> {
        // Собираем уникальные индексы семейств очередей
        let mut unique_families = vec![indices.graphics, indices.present, indices.transfer];
        unique_families.sort_unstable();
        unique_families.dedup();

        let queue_priorities = [1.0_f32];
        let queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = unique_families
            .iter()
            .map(|&family_index| {
                vk::DeviceQueueCreateInfo::default()
                .queue_family_index(family_index)
                .queue_priorities(&queue_priorities)
            })
            .collect();

        let device_extensions = [khr::swapchain::NAME.as_ptr()];
        let features = vk::PhysicalDeviceFeatures::default();

        let create_info = vk::DeviceQueueCreateInfo::default();
        let _ = create_info;

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extensions)
            .enabled_features(&features);

        let device = unsafe { instance.create_device(pdevice, &device_create_info, None)? };

        let graphics_queue = unsafe { device.get_device_queue(indices.graphics, 0) };
        let present_queue = unsafe { device.get_device_queue(indices.present, 0) };
        let transfer_queue = unsafe { device.get_device_queue(indices.transfer, 0) };

        Ok((device, graphics_queue, present_queue, transfer_queue))
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        // В Vulkan критически важен обратный порядок уничтожения ресурсов!
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
        println!("Vulkan корректно освободил все ресурсы.");
    }
}
