use crate::camera::FlyCamera;
use crate::settings::LevelSettings;
use bevy::gizmos::config::GizmoConfigGroup;
use bevy::prelude::*;
use big_space::prelude::CellCoord;

#[derive(Resource)]
pub struct GridShown;

#[derive(GizmoConfigGroup, Default, Debug, Reflect)]
pub struct EditorGizmos;

pub struct EditorGizmosPlugin;

impl Plugin for EditorGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<EditorGizmos>()
            .insert_resource(GridShown)
            .add_systems(Update, draw_editor_grid.run_if(resource_exists::<GridShown>));
    }
}

fn draw_editor_grid(
    mut gizmos: Gizmos<EditorGizmos>,
    settings: Res<LevelSettings>,
    cam_q: Query<&CellCoord, With<FlyCamera>>,
) {
    let chunk_size = settings.space.cell_edge_length;

    // Смещение глобального нуля (чанк 0,0,0) относительно текущей ячейки камеры
    let origin_offset = if let Ok(cam_cell) = cam_q.single() {
        Vec3::new(
            -(cam_cell.x as f32) * chunk_size,
            -(cam_cell.y as f32) * chunk_size,
            -(cam_cell.z as f32) * chunk_size,
        )
    } else {
        Vec3::ZERO
    };

    // 1. Оси мира в настоящем нуле (0, 0, 0)
    gizmos.linestrip(
        [origin_offset, origin_offset + Vec3::X * 2.0],
        Color::srgb(0.9, 0.2, 0.2),
    );
    gizmos.linestrip(
        [origin_offset, origin_offset + Vec3::Y * 2.0],
        Color::srgb(0.2, 0.9, 0.2),
    );
    gizmos.linestrip(
        [origin_offset, origin_offset + Vec3::Z * 2.0],
        Color::srgb(0.2, 0.2, 0.9),
    );

    // 2. Локальная метровая сетка 64x64 в нулевом чанке
    let half = 32.0;
    let mut x = -half;
    while x <= half {
        let c = if x == 0.0 {
            Color::srgba(0.7, 0.7, 0.7, 0.5)
        } else {
            Color::srgba(0.4, 0.4, 0.4, 0.2)
        };
        gizmos.linestrip(
            [
                origin_offset + Vec3::new(x, 0.0, -half),
                origin_offset + Vec3::new(x, 0.0, half),
            ],
            c,
        );
        x += 1.0;
    }
    let mut z = -half;
    while z <= half {
        let c = if z == 0.0 {
            Color::srgba(0.7, 0.7, 0.7, 0.5)
        } else {
            Color::srgba(0.4, 0.4, 0.4, 0.2)
        };
        gizmos.linestrip(
            [
                origin_offset + Vec3::new(-half, 0.0, z),
                origin_offset + Vec3::new(half, 0.0, z),
            ],
            c,
        );
        z += 1.0;
    }

    // Границы всего мира и сетка чанков (привязаны к настоящему нулю)
    let [grid_x, grid_z] = settings.space.world_grid_size;
    let total_x = grid_x as f32 * chunk_size;
    let total_z = grid_z as f32 * chunk_size;
    let half_x = total_x * 0.5;
    let half_z = total_z * 0.5;

    let border_color = Color::srgb(1.0, 0.45, 0.1);
    let y = 0.05;
    let p1 = origin_offset + Vec3::new(-half_x, y, -half_z);
    let p2 = origin_offset + Vec3::new(half_x, y, -half_z);
    let p3 = origin_offset + Vec3::new(half_x, y, half_z);
    let p4 = origin_offset + Vec3::new(-half_x, y, half_z);
    gizmos.linestrip([p1, p2, p3, p4, p1], border_color);

    let chunk_color = Color::srgba(0.2, 0.7, 1.0, 0.35);
    for ix in 1..grid_x {
        let cx = -half_x + ix as f32 * chunk_size;
        gizmos.linestrip(
            [
                origin_offset + Vec3::new(cx, y, -half_z),
                origin_offset + Vec3::new(cx, y, half_z),
            ],
            chunk_color,
        );
    }
    for iz in 1..grid_z {
        let cz = -half_z + iz as f32 * chunk_size;
        gizmos.linestrip(
            [
                origin_offset + Vec3::new(-half_x, y, cz),
                origin_offset + Vec3::new(half_x, y, cz),
            ],
            chunk_color,
        );
    }
}
