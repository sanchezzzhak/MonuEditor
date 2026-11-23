//! Monu Editor — редактор уровней на базе Monu Engine.
//!
//! Запуск: `cargo run -p monu_editor`
//! Управление камерой: ПКМ + WASD (как в Unreal Engine), Shift — ускорение.

use bevy::prelude::*;
use monu_engine::{app::AppConfig, MonuApp};

fn main() {
    let mut app = MonuApp::new(AppConfig::Editor);
    app.add_plugins(EditorPlugin);
    app.run();
}

/// Плагин редактора: пока минимальный каркас.
/// Дальше сюда: hierarchy-панель, инспектор (egui), selection, gizmo-манипуляторы.
struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, editor_startup_info)
            .add_systems(Update, hotkeys);
    }
}

fn editor_startup_info() {
    info!("Monu Editor запущен. Следующие шаги: панель иерархии, инспектор, выделение объектов мышью.");
}

fn hotkeys(keys: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
