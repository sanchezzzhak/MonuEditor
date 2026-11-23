//! Экспериментальный низкоуровневый Vulkan-контекст поверх ash.
//! Включается фичей `vulkan_raw`. В будущем — кастомные рендер-проходы
//! (deferred/GI), недоступные штатно через wgpu-пайплайн Bevy.

// TODO: instance -> device -> swapchain поверх surface, созданного Bevy/winit.
