pub mod heightmap;
pub mod sculpt;

pub use heightmap::TerrainHeightmap;

use crate::app::WorldGridRoot;
use crate::settings::LevelSettings;
use crate::world::LevelObject;
use bevy::prelude::*;
use big_space::prelude::CellCoord;

#[derive(Component)]
pub struct TerrainChunk {
    pub coord: IVec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainBrushMode {
    Raise,
    Lower,
    Smooth,
    Flatten,
}

#[derive(Resource, Debug, Clone)]
pub struct TerrainBrush {
    pub enabled: bool,
    pub mode: TerrainBrushMode,
    pub radius: f32,
    pub strength: f32,
    pub falloff: f32,
    pub target_height: f32,
    pub pointer_over_ui: bool
}

impl Default for TerrainBrush {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: TerrainBrushMode::Raise,
            radius: 25.0,
            strength: 15.0,
            falloff: 1.0,
            target_height: 0.0,
            pointer_over_ui: false,
        }
    }
}

/// Команды управления ландшафтом из UI
#[derive(Message, Debug, Clone)]
pub enum TerrainCommand {
    /// Создать чанк ландшафта в заданной секции (например, [0, 0])
    CreateChunk { coord: IVec2, resolution: usize },
    /// Сгенерировать тестовый холмистый рельеф (первичный шум до интеграции Nortantis)
    GenerateHills { seed: u32, amplitude: f32 },
    /// Сбросить высоты в 0
    ResetFlat,
}

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainBrush>()
            .add_message::<TerrainCommand>()
            .add_systems(
                Update,
                (handle_terrain_commands, sculpt::update_terrain_sculpting),
            );
    }
}

fn handle_terrain_commands(
    mut commands: Commands,
    mut reader: MessageReader<TerrainCommand>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<LevelSettings>,
    grid_root: Query<Entity, With<WorldGridRoot>>,
    mut existing_q: Query<(Entity, &mut TerrainHeightmap, &Mesh3d), With<TerrainChunk>>,
) {
    let Ok(root_entity) = grid_root.single() else { return };

    for cmd in reader.read() {
        match cmd {
            TerrainCommand::CreateChunk { coord, resolution } => {
                // Удаляем старый чанк в этой же координате, если был
                for (ent, _, _) in &existing_q {
                    commands.entity(ent).despawn();
                }

                let hm = TerrainHeightmap::new_flat(
                    *resolution,
                    settings.space.cell_edge_length,
                    300.0,
                );
                let mesh = meshes.add(hm.build_mesh());
                let mat = materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    perceptual_roughness: 0.9,
                    reflectance: 0.1,
                    ..default()
                });

                let terrain_ent = commands
                    .spawn((
                        Name::new(format!("Terrain_[{}, {}]", coord.x, coord.y)),
                        LevelObject,
                        TerrainChunk { coord: *coord },
                        hm,
                        Mesh3d(mesh),
                        MeshMaterial3d(mat),
                        Transform::default(),
                        CellCoord::new(coord.x as i64, 0, coord.y as i64),
                    ))
                    .id();

                commands.entity(root_entity).add_child(terrain_ent);
                info!("Создан чанк ландшафта [{}, {}]", coord.x, coord.y);
            }

            TerrainCommand::GenerateHills { seed, amplitude } => {
                for (_, mut hm, mesh_handle) in &mut existing_q {
                    let res = hm.resolution;
                    for z in 0..res {
                        for x in 0..res {
                            let p = hm.grid_to_local(x, z) * 0.006;
                            let s = *seed as f32 * 0.13;
                            // Многослойные волны (fBm-аппроксимация без внешних крейтов)
                            let h1 = ((p.x + s).sin() * (p.y - s).cos()) * 0.6;
                            let h2 = ((p.x * 2.3 - s).cos() * (p.y * 2.1 + s).sin()) * 0.25;
                            let h3 = ((p.x * 5.7 + s).sin() + (p.y * 5.3 - s).cos()) * 0.08;
                            let height = (h1 + h2 + h3) * amplitude;
                            hm.set(x, z, height);
                        }
                    }
                    if let Some(mut mesh) = meshes.get_mut(&mesh_handle.0) {
                        *mesh = hm.build_mesh();
                    }
                }
            }

            TerrainCommand::ResetFlat => {
                for (_, mut hm, mesh_handle) in &mut existing_q {
                    hm.heights.fill(0.0);
                    if let Some(mut mesh) = meshes.get_mut(&mesh_handle.0) {
                        *mesh = hm.build_mesh();
                    }
                }
            }
        }
    }
}
