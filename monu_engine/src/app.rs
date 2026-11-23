//! Фабрика приложений Monu Engine.
//!
//! И редактор, и игра строятся на одной конфигурации — чтобы сцена,
//! открытая в редакторе, выглядела в игре точно так же.

use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::reflect::Reflect;
use bevy::gizmos::config::{GizmoConfigGroup};

use crate::camera::EditorCameraPlugin;
use crate::world::WorldPlugin;
use crate::ENGINE_VERSION;

/// Режим запуска приложения
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppConfig {
    /// Полноценная игра
    Game,
    /// Редактор: полётная камера, гизмо, сетка
    Editor,
}

/// Конструктор приложений на базе движка
pub struct MonuApp;

impl MonuApp {
    /// Создать приложение в выбранном режиме.
    pub fn new(mode: AppConfig) -> App {
        let mut app = App::new();

        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: format!("Monu Engine v{ENGINE_VERSION} [{mode:?}]"),
                        resolution: (1600, 900).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    level: Level::INFO,
                    filter: "wgpu=error,naga=warn".into(),
                    ..default()
                }),
        );

        app.insert_resource(ModeResource(mode));
        app.add_plugins((WorldPlugin, EditorCameraPlugin));

        match mode {
            AppConfig::Game => Self::setup_game(&mut app),
            AppConfig::Editor => Self::setup_editor(&mut app),
        }

        app
    }

    fn setup_game(app: &mut App) {
        app.add_systems(Startup, spawn_default_level);
    }

    fn setup_editor(app: &mut App) {
        app.init_gizmo_group::<EditorGizmos>()
            .add_systems(Startup, spawn_editor_scene)
            .add_systems(Update, draw_editor_grid.run_if(resource_exists::<GridShown>));
    }
}

/// Ресурс с режимом работы
#[derive(Resource, Debug, Clone, Copy)]
pub struct ModeResource(pub AppConfig);

/// Ресурс-переключатель видимости сетки (будет дёргаться из UI редактора)
#[derive(Resource)]
pub struct GridShown;

fn spawn_default_level(mut commands: Commands) {
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 30000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
    ));
    info!("Default level spawned");
}

fn spawn_editor_scene(mut commands: Commands, config: Res<ModeResource>) {
    // Полётная камера редактора
    commands.spawn((
        Name::new("EditorCamera"),
        crate::camera::FlyCamera::default(),
        Transform::from_xyz(0.0, 2.0, 8.0),
        Camera3d::default(),
    ));

    commands.insert_resource(GridShown);
    info!("Editor scene ready ({:?})", config.0);
}

/// Отдельная группа гизмо для редактора (можно вкл/выкл независимо от игры)
#[derive(GizmoConfigGroup, Default, Debug, Reflect)]
struct EditorGizmos;

fn draw_editor_grid(mut gizmos: Gizmos<EditorGizmos>) {
    // Оси мира: X красный, Y зелёный, Z синий
    gizmos.linestrip(
        [Vec3::ZERO, Vec3::X * 2.0],
        Color::srgb(0.9, 0.2, 0.2),
    );
    gizmos.linestrip(
        [Vec3::ZERO, Vec3::Y * 2.0],
        Color::srgb(0.2, 0.9, 0.2),
    );
    gizmos.linestrip(
        [Vec3::ZERO, Vec3::Z * 2.0],
        Color::srgb(0.2, 0.2, 0.9),
    );

    // Сетка-пол 64x64
    let half = 32.0;
    let step = 1.0;
    let mut x = -half;
    while x <= half {
        let c = if x == 0.0 {
            Color::srgba(0.7, 0.7, 0.7, 0.5)
        } else {
            Color::srgba(0.4, 0.4, 0.4, 0.25)
        };
        gizmos.linestrip([Vec3::new(x, 0.0, -half), Vec3::new(x, 0.0, half)], c);
        x += step;
    }
    let mut z = -half;
    while z <= half {
        let c = if z == 0.0 {
            Color::srgba(0.7, 0.7, 0.7, 0.5)
        } else {
            Color::srgba(0.4, 0.4, 0.4, 0.25)
        };
        gizmos.linestrip([Vec3::new(-half, 0.0, z), Vec3::new(half, 0.0, z)], c);
        z += step;
    }
}

