pub mod heightmap;
pub mod sculpt;

pub use heightmap::TerrainHeightmap;

use crate::app::WorldGridRoot;
use crate::settings::LevelSettings;
use crate::world::LevelObject;
use bevy::prelude::*;
use big_space::prelude::CellCoord;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const TERRAIN_DIR: &str = "world_terrain";

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
    pub pointer_over_ui: bool,
}

impl Default for TerrainBrush {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: TerrainBrushMode::Raise,
            radius: 35.0,
            strength: 20.0,
            falloff: 1.0,
            target_height: 0.0,
            pointer_over_ui: false,
        }
    }
}

#[derive(Message, Debug, Clone)]
pub enum TerrainCommand {
    CreateChunk { coord: IVec2, resolution: usize },
    GenerateHills { seed: u32, amplitude: f32 },
    ResetFlat,
    StitchAllSeams,
    SaveAllChunks,
    LoadAllChunks,
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

fn spawn_chunk_entity(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root_entity: Entity,
    coord: IVec2,
    hm: TerrainHeightmap,
) {
    let mesh = meshes.add(hm.build_mesh());
    let mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.9,
        reflectance: 0.1,
        ..default()
    });

    let ent = commands
        .spawn((
            Name::new(format!("Terrain_[{}, {}]", coord.x, coord.y)),
            LevelObject,
            TerrainChunk { coord },
            hm,
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform::default(),
            CellCoord::new(coord.x as i64, 0, coord.y as i64),
        ))
        .id();

    commands.entity(root_entity).add_child(ent);
}

fn handle_terrain_commands(
    mut commands: Commands,
    mut reader: MessageReader<TerrainCommand>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<LevelSettings>,
    grid_root: Query<Entity, With<WorldGridRoot>>,
    mut chunks_q: Query<(Entity, &TerrainChunk, &mut TerrainHeightmap, &Mesh3d)>,
) {
    let Ok(root_entity) = grid_root.single() else { return };

    for cmd in reader.read() {
        match cmd {
            TerrainCommand::CreateChunk { coord, resolution } => {
                for (ent, chunk, _, _) in &chunks_q {
                    if chunk.coord == *coord {
                        commands.entity(ent).despawn();
                    }
                }

                let mut hm = TerrainHeightmap::new_flat(
                    *resolution,
                    settings.space.cell_edge_length,
                    300.0,
                );

                stitch_new_chunk_with_neighbors(*coord, &mut hm, &chunks_q);

                spawn_chunk_entity(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    root_entity,
                    *coord,
                    hm,
                );
                info!("Создан и пристыкован чанк [{}, {}]", coord.x, coord.y);
            }

            TerrainCommand::GenerateHills { seed, amplitude } => {
                let chunk_size = settings.space.cell_edge_length;
                for (_, chunk, mut hm, mesh_handle) in &mut chunks_q {
                    let res = hm.resolution;
                    let world_offset = Vec2::new(
                        chunk.coord.x as f32 * chunk_size,
                        chunk.coord.y as f32 * chunk_size,
                    );

                    for z in 0..res {
                        for x in 0..res {
                            let p = (hm.grid_to_local(x, z) + world_offset) * 0.004;
                            let s = *seed as f32 * 0.13;
                            let h1 = ((p.x + s).sin() * (p.y - s).cos()) * 0.6;
                            let h2 = ((p.x * 2.3 - s).cos() * (p.y * 2.1 + s).sin()) * 0.25;
                            let h3 = ((p.x * 5.7 + s).sin() + (p.y * 5.3 - s).cos()) * 0.08;
                            hm.set(x, z, (h1 + h2 + h3) * amplitude);
                        }
                    }

                    if let Some(mut mesh) = meshes.get_mut(&mesh_handle.0) {
                        *mesh = hm.build_mesh();
                    }
                }
            }

            TerrainCommand::ResetFlat => {
                for (_, _, mut hm, mesh_handle) in &mut chunks_q {
                    hm.heights.fill(0.0);
                    if let Some(mut mesh) = meshes.get_mut(&mesh_handle.0) {
                        *mesh = hm.build_mesh();
                    }
                }
            }

            TerrainCommand::StitchAllSeams => {
                stitch_all_loaded_chunks(&mut chunks_q, &mut meshes);
                info!("Швы всех чанков синхронизированы");
            }

            TerrainCommand::SaveAllChunks => {
                if let Err(e) = fs::create_dir_all(TERRAIN_DIR) {
                    error!("Не удалось создать папку {TERRAIN_DIR}: {e}");
                    continue;
                }
                let mut count = 0;
                for (_, chunk, hm, _) in &chunks_q {
                    let path = format!("{TERRAIN_DIR}/chunk_{}_{}.bin", chunk.coord.x, chunk.coord.y);
                    if let Err(e) = save_heightmap_binary(&path, hm) {
                        error!("Ошибка сохранения {path}: {e}");
                    } else {
                        count += 1;
                    }
                }
                info!("Сохранено чанков ландшафта: {count}");
            }

            TerrainCommand::LoadAllChunks => {
                let Ok(entries) = fs::read_dir(TERRAIN_DIR) else {
                    warn!("Папка {TERRAIN_DIR} не найдена");
                    continue;
                };

                for (ent, _, _, _) in &chunks_q {
                    commands.entity(ent).despawn();
                }

                let mut count = 0;
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) != Some("bin") {
                        continue;
                    }
                    if let Some(coord) = parse_chunk_coord_from_filename(&path) {
                        if let Ok(hm) = load_heightmap_binary(&path, settings.space.cell_edge_length) {
                            spawn_chunk_entity(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                root_entity,
                                coord,
                                hm,
                            );
                            count += 1;
                        }
                    }
                }
                info!("Загружено чанков ландшафта: {count}");
            }
        }
    }
}

fn stitch_new_chunk_with_neighbors(
    coord: IVec2,
    new_hm: &mut TerrainHeightmap,
    chunks_q: &Query<(Entity, &TerrainChunk, &mut TerrainHeightmap, &Mesh3d)>,
) {
    let res = new_hm.resolution;
    for (_, other_chunk, other_hm, _) in chunks_q {
        if other_hm.resolution != res {
            continue;
        }
        let diff = other_chunk.coord - coord;
        if diff == IVec2::new(1, 0) {
            for z in 0..res {
                new_hm.set(res - 1, z, other_hm.get(0, z));
            }
        }
        if diff == IVec2::new(-1, 0) {
            for z in 0..res {
                new_hm.set(0, z, other_hm.get(res - 1, z));
            }
        }
        if diff == IVec2::new(0, 1) {
            for x in 0..res {
                new_hm.set(x, res - 1, other_hm.get(x, 0));
            }
        }
        if diff == IVec2::new(0, -1) {
            for x in 0..res {
                new_hm.set(x, 0, other_hm.get(x, res - 1));
            }
        }
    }
}

fn stitch_all_loaded_chunks(
    chunks_q: &mut Query<(Entity, &TerrainChunk, &mut TerrainHeightmap, &Mesh3d)>,
    meshes: &mut Assets<Mesh>,
) {
    // 1. Сохраняем снимки высот и разрешений всех загруженных чанков по их координатам
    let mut map: HashMap<IVec2, Vec<f32>> = HashMap::new();
    let mut resolutions: HashMap<IVec2, usize> = HashMap::new();

    for (_, chunk, hm, _) in chunks_q.iter() {
        map.insert(chunk.coord, hm.heights.clone());
        resolutions.insert(chunk.coord, hm.resolution);
    }

    // 2. Проходим по каждому чанку и усредняем граничные вершины с 4 соседями
    for (_, chunk, mut hm, mesh_handle) in chunks_q.iter_mut() {
        let res = hm.resolution;
        let mut changed = false;

        // Правый сосед (+X): наш край x = res - 1 стыкуется с его краем x = 0
        let right_coord = chunk.coord + IVec2::new(1, 0);
        if let (Some(right_h), Some(&r_res)) = (map.get(&right_coord), resolutions.get(&right_coord)) {
            if r_res == res {
                for z in 0..res {
                    let avg = (hm.get(res - 1, z) + right_h[z * res]) * 0.5;
                    hm.set(res - 1, z, avg);
                }
                changed = true;
            }
        }

        // Левый сосед (-X): наш край x = 0 стыкуется с его краем x = res - 1
        let left_coord = chunk.coord + IVec2::new(-1, 0);
        if let (Some(left_h), Some(&l_res)) = (map.get(&left_coord), resolutions.get(&left_coord)) {
            if l_res == res {
                for z in 0..res {
                    let avg = (hm.get(0, z) + left_h[z * res + (res - 1)]) * 0.5;
                    hm.set(0, z, avg);
                }
                changed = true;
            }
        }

        // Нижний сосед (+Z): наш край z = res - 1 стыкуется с его краем z = 0
        let bottom_coord = chunk.coord + IVec2::new(0, 1);
        if let (Some(bot_h), Some(&b_res)) = (map.get(&bottom_coord), resolutions.get(&bottom_coord)) {
            if b_res == res {
                for x in 0..res {
                    let avg = (hm.get(x, res - 1) + bot_h[x]) * 0.5;
                    hm.set(x, res - 1, avg);
                }
                changed = true;
            }
        }

        // Верхний сосед (-Z): наш край z = 0 стыкуется с его краем z = res - 1
        let top_coord = chunk.coord + IVec2::new(0, -1);
        if let (Some(top_h), Some(&t_res)) = (map.get(&top_coord), resolutions.get(&top_coord)) {
            if t_res == res {
                for x in 0..res {
                    let avg = (hm.get(x, 0) + top_h[(res - 1) * res + x]) * 0.5;
                    hm.set(x, 0, avg);
                }
                changed = true;
            }
        }

        // 3. Если хотя бы одна граница изменилась — пересобираем 3D-меш чанка
        if changed {
            if let Some(mut mesh) = meshes.get_mut(&mesh_handle.0) {
                *mesh = hm.build_mesh();
            }
        }
    }
}

fn save_heightmap_binary(path: &str, hm: &TerrainHeightmap) -> std::io::Result<()> {
    let mut bytes = Vec::with_capacity(8 + hm.heights.len() * 4);
    bytes.extend_from_slice(&(hm.resolution as u32).to_le_bytes());
    bytes.extend_from_slice(&hm.max_height.to_le_bytes());
    for &h in &hm.heights {
        bytes.extend_from_slice(&h.to_le_bytes());
    }
    fs::write(path, bytes)
}
fn load_heightmap_binary(path: &Path, size_meters: f32) -> Result<TerrainHeightmap, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    if bytes.len() < 8 {
        return Err("Файл повреждён".into());
    }
    let res = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
    let max_height = f32::from_le_bytes(bytes[4..8].try_into().unwrap());
    let expected_len = 8 + res * res * 4;
    if bytes.len() != expected_len {
        return Err("Несовпадение размера сетки".into());
    }
    let mut heights = Vec::with_capacity(res * res);
    for chunk in bytes[8..].chunks_exact(4) {
        heights.push(f32::from_le_bytes(chunk.try_into().unwrap()));
    }
    Ok(TerrainHeightmap {
        resolution: res,
        size_meters,
        max_height,
        heights,
    })
}
fn parse_chunk_coord_from_filename(path: &Path) -> Option<IVec2> {
    let stem = path.file_stem()?.to_str()?; // Например, "chunk_-1_2"
    let parts: Vec<&str> = stem.split('_').collect();

    match parts.as_slice() {
        ["chunk", x_str, z_str] => {
            let x = x_str.parse::<i32>().ok()?;
            let z = z_str.parse::<i32>().ok()?;
            Some(IVec2::new(x, z))
        }
        _ => None,
    }
}

