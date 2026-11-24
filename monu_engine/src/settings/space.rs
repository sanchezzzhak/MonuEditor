use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Настройки большого пространства (196 км² и плавающий центр координат)
#[derive(Reflect, Serialize, Deserialize, Debug, Clone)]
pub struct WorldSpaceSettings {
    pub cell_edge_length: f32,
    pub switching_threshold: f32,
    pub world_grid_size: [u32; 2],
    pub streaming_radius: u32,
}

impl Default for WorldSpaceSettings {
    fn default() -> Self {
        Self {
            cell_edge_length: 1000.0,
            switching_threshold: 500.0,
            world_grid_size:[14, 14],
            streaming_radius: 3,
        }
    }
}

impl WorldSpaceSettings {
    /// Общая площадь мира в км²
    pub fn total_area_km2(&self) -> f32 {
        let width_km = (self.world_grid_size[0] as f32 * self.cell_edge_length) / 1000.0;
        let depth_km = (self.world_grid_size[1] as f32 * self.cell_edge_length) / 1000.0;
        width_km * depth_km
    }
}
