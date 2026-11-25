use super::heightmap::TerrainHeightmap;
use super::{TerrainBrush, TerrainBrushMode, TerrainChunk};
use crate::camera::FlyCamera;
use crate::gizmos::EditorGizmos;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// Обновление визуального кольца кисти и лепка рельефа по ЛКМ
pub fn update_terrain_sculpting(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    brush: Res<TerrainBrush>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    cam_q: Query<(&Camera, &GlobalTransform), With<FlyCamera>>,
    mut terrain_q: Query<(&GlobalTransform, &mut TerrainHeightmap, &Mesh3d), With<TerrainChunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut gizmos: Gizmos<EditorGizmos>,
) {
    // Если кисть выключена ИЛИ курсор сейчас над панелью UI — ничего не делаем
    if !brush.enabled || brush.pointer_over_ui {
        return;
    }

    let Ok(window) = window_q.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, cam_gtf)) = cam_q.single() else { return };
    let Ok(ray) = camera.viewport_to_world(cam_gtf, cursor_pos) else { return };

    for (terrain_gtf, mut heightmap, mesh_handle) in &mut terrain_q {
        // Переводим луч в локальные координаты чанка (работает при любом Floating Origin!)
        let inv_mat = terrain_gtf.to_matrix().inverse();
        let local_origin = inv_mat.transform_point3(ray.origin);
        let local_dir = inv_mat.transform_vector3(*ray.direction).normalize_or_zero();

        let Some(hit_local) = heightmap.raycast_local(local_origin, local_dir) else {
            continue;
        };

        // Рисуем кольцо кисти на поверхности ландшафта
        let hit_world = terrain_gtf.transform_point(hit_local) + Vec3::Y * 0.15;
        let isometry = Isometry3d::new(hit_world, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
        gizmos.circle(isometry, brush.radius, Color::srgb(0.2, 0.95, 0.5));

        // Применяем кисть при зажатой ЛКМ (и если не зажата ПКМ для полёта камеры)
        if mouse.pressed(MouseButton::Left) && !mouse.pressed(MouseButton::Right) {
            let changed = apply_brush_stroke(
                &mut heightmap,
                Vec2::new(hit_local.x, hit_local.z),
                &brush,
                time.delta_secs(),
            );

            if changed {
                if let Some(mut mesh) = meshes.get_mut(&mesh_handle.0) {
                    *mesh = heightmap.build_mesh();
                }
            }
        }
    }
}

fn apply_brush_stroke(
    hm: &mut TerrainHeightmap,
    center: Vec2,
    brush: &TerrainBrush,
    dt: f32,
) -> bool {
    let res = hm.resolution;
    let step = hm.size_meters / (res - 1) as f32;
    let half = hm.size_meters * 0.5;
    let r2 = brush.radius * brush.radius;

    let min_x = (((center.x - brush.radius + half) / step).floor() as isize).clamp(0, (res - 1) as isize) as usize;
    let max_x = (((center.x + brush.radius + half) / step).ceil() as isize).clamp(0, (res - 1) as isize) as usize;
    let min_z = (((center.y - brush.radius + half) / step).floor() as isize).clamp(0, (res - 1) as isize) as usize;
    let max_z = (((center.y + brush.radius + half) / step).ceil() as isize).clamp(0, (res - 1) as isize) as usize;

    let mut modified = false;

    // Для режима Smooth предварительно копируем участок
    let snapshot = if matches!(brush.mode, TerrainBrushMode::Smooth) {
        Some(hm.heights.clone())
    } else {
        None
    };

    for z in min_z..=max_z {
        for x in min_x..=max_x {
            let pos = hm.grid_to_local(x, z);
            let dist_sq = pos.distance_squared(center);
            if dist_sq > r2 {
                continue;
            }

            // Плавное затухание от центра к краю кисти (Cosine falloff)
            let dist = dist_sq.sqrt();
            let norm_dist = (dist / brush.radius).clamp(0.0, 1.0);
            let falloff = (0.5 * (1.0 + (norm_dist * std::f32::consts::PI).cos())).powf(brush.falloff);
            let delta = brush.strength * falloff * dt;

            let cur = hm.get(x, z);
            let next = match brush.mode {
                TerrainBrushMode::Raise => cur + delta,
                TerrainBrushMode::Lower => cur - delta,
                TerrainBrushMode::Flatten => {
                    let diff = brush.target_height - cur;
                    cur + diff * (falloff * 5.0 * dt).clamp(0.0, 1.0)
                }
                TerrainBrushMode::Smooth => {
                    if let Some(ref snap) = snapshot {
                        let mut sum = 0.0;
                        let mut count = 0.0;
                        for oz in z.saturating_sub(1)..=(z + 1).min(res - 1) {
                            for ox in x.saturating_sub(1)..=(x + 1).min(res - 1) {
                                sum += snap[oz * res + ox];
                                count += 1.0;
                            }
                        }
                        let avg = sum / count;
                        cur + (avg - cur) * (falloff * 8.0 * dt).clamp(0.0, 1.0)
                    } else {
                        cur
                    }
                }
            };

            hm.set(x, z, next);
            modified = true;
        }
    }

    modified
}
