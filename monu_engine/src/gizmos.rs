use crate::settings::LevelSettings;
use bevy::gizmos::config::GizmoConfigGroup;
use bevy::prelude::*;

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

fn draw_editor_grid(mut gizmos: Gizmos<EditorGizmos>, settings: Res<LevelSettings>) {
    // 1. Оси мира
    gizmos.linestrip([Vec3::ZERO, Vec3::X * 2.0], Color::srgb(0.9, 0.2, 0.2));
    gizmos.linestrip([Vec3::ZERO, Vec3::Y * 2.0], Color::srgb(0.2, 0.9, 0.2));
    gizmos.linestrip([Vec3::ZERO, Vec3::Z * 2.0], Color::srgb(0.2, 0.2, 0.9));

    // 2. Локальная метровая сетка 64x64
    let half = 32.0;
    let mut x = -half;
    while x <= half {
        let c = if x == 0.0 {
            Color::srgba(0.7, 0.7, 0.7, 0.5)
        } else {
            Color::srgba(0.4, 0.4, 0.4, 0.2)
        };
        gizmos.linestrip([Vec3::new(x, 0.0, -half), Vec3::new(x, 0.0, half)], c);
        x += 1.0;
    }
    let mut z = -half;
    while z <= half {
        let c = if z == 0.0 {
            Color::srgba(0.7, 0.7, 0.7, 0.5)
        } else {
            Color::srgba(0.4, 0.4, 0.4, 0.2)
        };
        gizmos.linestrip([Vec3::new(-half, 0.0, z), Vec3::new(half, 0.0, z)], c);
        z += 1.0;
    }

    // 3. Границы всего мира и сетка чанков
    let chunk_size = settings.space.cell_edge_length;
    let total_x = settings.space.world_grid_size[0] as f32 * chunk_size;
    let total_z = settings.space.world_grid_size[1] as f32 * chunk_size;
    let half_x = total_x * 0.5;
    let half_z = total_z * 0.5;

    let border_color = Color::srgb(1.0, 0.45, 0.1);
    let y = 0.05;
    let p1 = Vec3::new(-half_x, y, -half_z);
    let p2 = Vec3::new(half_x, y, -half_z);
    let p3 = Vec3::new(half_x, y, half_z);
    let p4 = Vec3::new(-half_x, y, half_z);
    gizmos.linestrip([p1, p2, p3, p4, p1], border_color);

    let chunk_color = Color::srgba(0.2, 0.7, 1.0, 0.35);
    for ix in 1..settings.space.world_grid_size[0] {
        let cx = -half_x + ix as f32 * chunk_size;
        gizmos.linestrip([Vec3::new(cx, y, -half_z), Vec3::new(cx, y, half_z)], chunk_color);
    }
    for iz in 1..settings.space.world_grid_size[1] {
        let cz = -half_z + iz as f32 * chunk_size;
        gizmos.linestrip([Vec3::new(-half_x, y, cz), Vec3::new(half_x, y, cz)], chunk_color);
    }
}
