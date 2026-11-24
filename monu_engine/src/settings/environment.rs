use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Настройки окружения и физики текущего уровня
#[derive(Reflect, Serialize, Deserialize, Debug, Clone)]
pub struct EnvironmentSettings {
    pub sky_color: [f32; 3],
    pub ambient_brightness: f32,
    pub sun_pitch_deg: f32,
    pub sun_yaw_deg: f32,
    pub sun_illuminance: f32,
    pub fog_enabled: bool,
    pub fog_start: f32,
    pub fog_end: f32,
    pub gravity: f32,
}

impl Default for EnvironmentSettings {
    fn default() -> Self {
        Self {
            sky_color: [0.45, 0.65, 0.88],
            ambient_brightness: 300.0,
            sun_pitch_deg: -45.0,
            sun_yaw_deg: 35.0,
            sun_illuminance: 15_000.0,
            fog_enabled: true,
            fog_start: 800.0,
            fog_end: 3000.0,
            gravity: -9.81,
        }
    }
}
