pub mod vulkan_ctx;

use std::sync::Arc;
use std::time::Instant;

use vulkan_ctx::{VulkanContext};

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// Хуки для расширения движка (используется редактором для перехвата ввода и отрисовки UI)
pub trait EngineHooks {
    /// Вызывается один раз после создания окна (здесь редактор инициализирует egui)
    fn on_init(&mut self, _window: &Window) {}

    /// Перехват событий окна. Если возвращает true — событие поглощено интерфейсом (например, клик по кнопке UI)
    fn on_window_event(&mut self, _window: &Window, _event: &WindowEvent) -> bool {
        false
    }

    /// Вызывается каждый кадр перед финальной отправкой кадра на экран
    fn on_render_ui(&mut self, _window: &Window) {}
}

/// Пустая реализация для чистой игры (компилятор полностью вырежет вызовы)
pub struct EmptyHooks;
impl EngineHooks for EmptyHooks {}

pub struct EngineConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            title: "MonuEngine".to_string(),
            width: 1280,
            height: 720,
        }
    }
}

pub struct EngineRunner<H: EngineHooks> {
    config: EngineConfig,
    vk_ctx: Option<VulkanContext>,
    window: Option<Arc<Window>>,
    hooks: H,
    last_frame: Instant,
}

impl<H: EngineHooks> EngineRunner<H> {
    pub fn run(config: EngineConfig, hooks: H) -> anyhow::Result<()> {
        let event_loop = EventLoop::new()?;
        // Poll обеспечивает максимальный FPS; позже для редактора в простое можно переключать на Wait
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut runner = Self {
            config,
            window: None,
            vk_ctx: None,
            hooks,
            last_frame: Instant::now(),
        };

        event_loop.run_app(&mut runner)?;
        Ok(())
    }
}

impl<H: EngineHooks> ApplicationHandler for EngineRunner<H> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes()
                .with_title(&self.config.title)
                .with_inner_size(winit::dpi::LogicalSize::new(
                    self.config.width,
                    self.config.height,
                ));

            let window = Arc::new(
                event_loop
                .create_window(attrs)
                .expect("Не удалось создать окно"),
            );


            let vk_ctx = VulkanContext::new(&window)
                .expect("Критическая ошибка инициализации Vulkan");
            
            self.hooks.on_init(&window);
            self.vk_ctx = Some(vk_ctx);
            self.window = Some(window);
            self.last_frame = Instant::now();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = &self.window else { return };
        if window.id() != window_id {
            return;
        }

        // Сначала даем редактору (egui) обработать событие
        let consumed = self.hooks.on_window_event(window, &event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    // TODO: Пересоздание Vulkan Swapchain
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let _dt = (now - self.last_frame).as_secs_f32();
                self.last_frame = now;

                // 1. TODO: Забрать свежий RenderPacket из потока игровой логики
                // 2. TODO: Записать команды Vulkan для 3D-сцены и ландшафта
                // 3. Отрисовка интерфейса редактора (если подключен)
                self.hooks.on_render_ui(window);
                // 4. TODO: Present кадра

                window.request_redraw();
            }
            _ => {
                if !consumed {
                    // TODO: Передать ввод (клавиатура/мышь) в камеру или игровую логику
                }
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        _event: DeviceEvent,
    ) {
        // Сырое движение мыши (Raw Mouse Motion) — пригодится для свободного полета камеры во вьюпорте
    }
}
