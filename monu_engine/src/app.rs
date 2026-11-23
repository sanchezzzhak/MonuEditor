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

use bevy::transform::TransformPlugin;
use big_space::prelude::*;
use crate::settings::{EditorCommand, WorldSpaceSettings};
use crate::ecs::Primitives;


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
        let space_settings = WorldSpaceSettings::default();

        app.add_plugins((
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
            })
            .disable::<TransformPlugin>(),
            BigSpaceDefaultPlugins,
        ))
            .insert_resource(ModeResource(mode))
            .insert_resource(space_settings)
            .register_type::<WorldSpaceSettings>()
            .add_message::<EditorCommand>()
            .add_plugins((WorldPlugin, EditorCameraPlugin))
            .add_systems(Update, handle_editor_commands);

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

#[derive(Component)]
pub struct WorldGridRoot;
fn spawn_editor_scene(
    mut commands: Commands,
    config: Res<ModeResource>,
    space: Res<WorldSpaceSettings>,
) {
    commands
        .spawn((
            WorldGridRoot,
            BigSpace::default(),
            Grid::new(space.cell_edge_length, space.switching_threshold),
        ))
        .with_children(|root| {
            root.spawn((
                Name::new("EditorCamera"),
                crate::camera::FlyCamera::default(),
                Transform::from_xyz(0.0, 5.0, 12.0),
                CellCoord::default(),
                FloatingOrigin,
                Camera3d::default(),
            ));

            root.spawn((
                Name::new("EditorSun"),
                DirectionalLight {
                    illuminance: 25000.0,
                    shadow_maps_enabled: true,
                    ..default()
                },
                Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
                CellCoord::default(),
            ));
        });

    commands.insert_resource(GridShown);
    info!("Editor scene ready ({:?})", config.0);
}

fn handle_editor_commands(
    mut commands: Commands,
    mut reader: MessageReader<EditorCommand>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    grid_root: Query<Entity, With<WorldGridRoot>>,
    grid_shown: Option<Res<GridShown>>,
) {
    let Ok(root_entity) = grid_root.single() else { return };

    for cmd in reader.read() {
        match cmd {
            EditorCommand::SpawnCube { name, size, pos } => {
                let entity = Primitives::cube(&mut commands, &mut meshes, &mut materials, name, *size, *pos)
                    .insert(CellCoord::default())
                    .id();
                commands.entity(root_entity).add_child(entity);
            }
            EditorCommand::SpawnPlane { size } => {
                let entity = Primitives::plane(&mut commands, &mut meshes, &mut materials, *size)
                    .insert(CellCoord::default())
                    .id();
                commands.entity(root_entity).add_child(entity);
            }
            EditorCommand::DeleteEntity(entity) => {
                commands.entity(*entity).despawn();
            }
            EditorCommand::ToggleGrid => {
                if grid_shown.is_some() {
                    commands.remove_resource::<GridShown>();
                } else {
                    commands.insert_resource(GridShown);
                }
            }
        }
    }
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

