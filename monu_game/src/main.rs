//! monu_game — демонстрационная игра на Monu Engine.
//!
//! Запуск: `cargo run -p monu_game`

use bevy::prelude::*;
use monu_engine::{app::AppConfig, MonuApp};

fn main() {
    let mut app = MonuApp::new(AppConfig::Game);
    app.add_systems(Startup, spawn_demo_objects);
    app.run();
}

fn spawn_demo_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Пол
    monu_engine::ecs::Primitives::plane(&mut commands, &mut meshes, &mut materials, 40.0);
    // Куб перед камерой
    monu_engine::ecs::Primitives::cube(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Cube",
        Vec3::ONE,
        Vec3::new(0.0, 0.5, 0.0),
    );
    // Игровая камера
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
