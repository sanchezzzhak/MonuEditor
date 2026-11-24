use crate::app::WorldGridRoot;
use crate::ecs::Primitives;
use crate::gizmos::GridShown;
use bevy::prelude::*;
use big_space::prelude::CellCoord;

/// Команды, которые UI редактора отправляет в ядро движка
#[derive(Message, Debug, Clone)]
pub enum EditorCommand {
    SpawnCube { name: String, size: Vec3, pos: Vec3 },
    SpawnPlane { size: f32 },
    DeleteEntity(Entity),
    ToggleGrid,
    SaveLevelSettings,
    ReloadLevelSettings,
}

pub struct EditorCommandsPlugin;

impl Plugin for EditorCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EditorCommand>()
            .add_systems(Update, handle_scene_commands);
    }
}

fn handle_scene_commands(
    mut commands: Commands,
    mut reader: MessageReader<EditorCommand>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    grid_root: Query<Entity, With<WorldGridRoot>>,
    grid_shown: Option<Res<GridShown>>,
) {
    let Ok(root_entity) = grid_root.single() else {
        return;
    };

    for cmd in reader.read() {
        match cmd {
            EditorCommand::SpawnCube { name, size, pos } => {
                let entity =
                    Primitives::cube(&mut commands, &mut meshes, &mut materials, name, *size, *pos)
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
            _ => {}
        }
    }
}
