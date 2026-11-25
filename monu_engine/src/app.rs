use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::transform::TransformPlugin;
use big_space::prelude::*;

use crate::camera::EditorCameraPlugin;
use crate::commands::EditorCommandsPlugin;
use crate::gizmos::EditorGizmosPlugin;
use crate::settings::{LevelSettings, LevelSettingsPlugin};
use crate::terrain::TerrainPlugin;
use crate::world::WorldPlugin;
use crate::ENGINE_VERSION;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppConfig {
    Game,
    Editor,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct ModeResource(pub AppConfig);

#[derive(Component)]
pub struct WorldGridRoot;

pub struct MonuApp;

impl MonuApp {
    pub fn new(mode: AppConfig) -> App {
        let mut app = App::new();

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
            .add_plugins((
                LevelSettingsPlugin,
                EditorCommandsPlugin,
                TerrainPlugin,
                WorldPlugin,
                EditorCameraPlugin,
            ));

        match mode {
            AppConfig::Game => {
                app.add_systems(Startup, spawn_default_level);
            }
            AppConfig::Editor => {
                app.add_plugins(EditorGizmosPlugin)
                    .add_systems(Startup, spawn_editor_scene);
            }
        }

        app
    }
}

fn spawn_default_level(mut commands: Commands) {
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 30000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
    ));
}

fn spawn_editor_scene(
    mut commands: Commands,
    config: Res<ModeResource>,
    settings: Res<LevelSettings>,
) {
    commands
        .spawn((
            WorldGridRoot,
            BigSpace::default(),
            Grid::new(
                settings.space.cell_edge_length,
                settings.space.switching_threshold,
            ),
        ))
        .with_children(|root| {
            root.spawn((
                Name::new("EditorCamera"),
                crate::camera::FlyCamera::default(),
                Transform::from_xyz(0.0, 5.0, 12.0),
                CellCoord::default(),
                FloatingOrigin,
                Camera3d::default(),
                DistanceFog::default(),
            ));

            root.spawn((
                Name::new("EditorSun"),
                DirectionalLight {
                    illuminance: settings.env.sun_illuminance,
                    shadow_maps_enabled: true,
                    ..default()
                },
                Transform::from_rotation(Quat::from_euler(
                    EulerRot::YXZ,
                    settings.env.sun_yaw_deg.to_radians(),
                    settings.env.sun_pitch_deg.to_radians(),
                    0.0,
                )),
                CellCoord::default(),
            ));
        });

    info!("Editor scene ready ({:?})", config.0);
}
