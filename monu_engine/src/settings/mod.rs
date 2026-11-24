pub mod environment;
pub mod space;
pub use environment::EnvironmentSettings;
pub use space::WorldSpaceSettings;

use crate::commands::EditorCommand;
use bevy::prelude::*;
use big_space::prelude::Grid;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const SETTINGS_FILE: &str = "level_settings.json";

/// Главный ресурс настроек уровня
#[derive(Resource, Reflect, Serialize, Deserialize, Debug, Clone)]
#[reflect(Resource)]
pub struct LevelSettings {
    pub level_name: String,
    pub space: WorldSpaceSettings,
    pub env: EnvironmentSettings,
}

impl Default for LevelSettings {
    fn default() -> Self {
        Self {
            level_name: "Main_World".to_string(),
            space: WorldSpaceSettings::default(),
            env: EnvironmentSettings::default(),
        }
    }
}

impl LevelSettings {
    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(parsed) = serde_json::from_str::<Self>(&data) {
                return parsed;
            }
        }
        Self::default()
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())
    }
}

pub struct LevelSettingsPlugin;

impl Plugin for LevelSettingsPlugin {
    fn build(&self, app: &mut App) {
        let loaded = LevelSettings::load_or_default(SETTINGS_FILE);
        app.register_type::<LevelSettings>()
            .insert_resource(loaded)
            .add_systems(Update, (apply_level_settings, handle_settings_commands));
    }
}

fn apply_level_settings(
    settings: Res<LevelSettings>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut sun_q: Query<(&mut DirectionalLight, &mut Transform)>,
    mut fog_q: Query<&mut DistanceFog>,
    mut grid_q: Query<&mut Grid>,
) {
    if !settings.is_changed() {
        return;
    }

    for mut grid in &mut grid_q {
        *grid = Grid::new(
            settings.space.cell_edge_length,
            settings.space.switching_threshold,
        );
    }

    let [r, g, b] = settings.env.sky_color;
    clear_color.0 = Color::srgb(r, g, b);
    ambient.brightness = settings.env.ambient_brightness;

    let pitch = settings.env.sun_pitch_deg.to_radians();
    let yaw = settings.env.sun_yaw_deg.to_radians();
    for (mut light, mut tf) in &mut sun_q {
        light.illuminance = settings.env.sun_illuminance;
        tf.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    }

    for mut fog in &mut fog_q {
        if settings.env.fog_enabled {
            fog.color = Color::srgb(r, g, b);
            fog.falloff = FogFalloff::Linear {
                start: settings.env.fog_start,
                end: settings.env.fog_end,
            };
        } else {
            fog.falloff = FogFalloff::Linear {
                start: 1_000_000.0,
                end: 1_000_001.0,
            };
        }
    }
}

fn handle_settings_commands(
    mut cmd_reader: MessageReader<EditorCommand>,
    mut settings: ResMut<LevelSettings>,
) {
    for cmd in cmd_reader.read() {
        match cmd {
            EditorCommand::SaveLevelSettings => {
                if let Err(err) = settings.save_to_file(SETTINGS_FILE) {
                    error!("Ошибка сохранения {SETTINGS_FILE}: {err}");
                } else {
                    info!("Настройки уровня сохранены в {SETTINGS_FILE}");
                }
            }
            EditorCommand::ReloadLevelSettings => {
                *settings = LevelSettings::load_or_default(SETTINGS_FILE);
                info!("Настройки уровня перезагружены");
            }
            _ => {}
        }
    }
}
