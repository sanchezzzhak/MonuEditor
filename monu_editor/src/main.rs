use monu_engine::{EngineConfig, EngineHooks, EngineRunner};
use winit::event::WindowEvent;
use winit::window::Window;

struct EditorApp {
    // Сюда скоро добавим egui::Context, egui_winit::State и egui_dock::DockState
}

impl EngineHooks for EditorApp {
    fn on_init(&mut self, _window: &Window) {
        println!("Редактор: окно готово, можно инициализировать egui");
    }

    fn on_window_event(&mut self, _window: &Window, _event: &WindowEvent) -> bool {
        // Здесь egui будет перехватывать клики по панелям
        false
    }

    fn on_render_ui(&mut self, _window: &Window) {
        // Здесь будем рисовать окна в стиле Blender/Unreal
    }
}

fn main() {
    let config = EngineConfig {
        title: "MonuEditor [Vulkan]".to_string(),
        width: 1600,
        height: 900,
    };

    let editor = EditorApp {};
    EngineRunner::run(config, editor).expect("Ошибка запуска редактора");
}
