use super::EditorUiState;
use bevy::prelude::*;
use monu_engine::commands::EditorCommand;
use monu_engine::settings::LevelSettings;
use monu_engine::terrain::{TerrainBrush, TerrainCommand};
use monu_engine::world::LevelObject;
use monu_engine::CellCoord;

/// Единый контекст редактора — передаётся во все панели и вкладки
pub struct EditorUiContext<'a, 'w1, 'w2, 'w3, 's, 'n, 't> {
    pub state: &'a mut EditorUiState,
    pub cmd: &'a mut MessageWriter<'w1, EditorCommand>,
    pub terrain_cmd: &'a mut MessageWriter<'w2, TerrainCommand>,
    pub settings: &'a mut LevelSettings,
    pub brush: &'a mut TerrainBrush,
    pub objects: &'a mut Query<'w3, 's, (Entity, &'n Name, &'t mut Transform), With<LevelObject>>,
    pub has_terrain: bool,
    pub cam_cell: CellCoord,
}
