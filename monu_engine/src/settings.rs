use bevy::prelude::*;

/// Настройки большого пространства (192 км² и плавающий центр координат)
#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct WorldSpaceSettings {
    /// Размер одного чанка (ячейки сетки) в метрах (по умолчанию 1000 м = 1 км)
    pub cell_edge_length: f32,
    /// Дистанция от центра ячейки, при которой срабатывает сдвиг Floating Origin
    pub switching_threshold: f32,
    /// Размер мира в чанках по X и Z (14 x 14 = 196 км²)
    pub world_grid_size: UVec2,
}

impl Default for WorldSpaceSettings {
    fn default() -> Self {
        Self {
            cell_edge_length: 1000.0,
            switching_threshold: 500.0,
            world_grid_size: UVec2::new(14, 14),
        }
    }
}

/// Команды, которые UI редактора отправляет в ядро движка
#[derive(Message, Debug, Clone)]
pub enum EditorCommand {
    SpawnCube { name: String, size: Vec3, pos: Vec3 },
    SpawnPlane { size: f32 },
    DeleteEntity(Entity),
    ToggleGrid,
}
