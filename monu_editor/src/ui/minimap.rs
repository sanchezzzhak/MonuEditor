use super::context::EditorUiContext;
use super::theme::MonuPalette;
use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, CornerRadius, Pos2, Rect, RichText, Sense, Stroke};
use monu_engine::terrain::{TerrainChunk, TerrainHeightmap};

pub fn draw_minimap_tab(
    ui: &mut egui::Ui,
    ctx: &mut EditorUiContext,
    chunks_q: &Query<(&TerrainChunk, &TerrainHeightmap)>,
) {
    let [grid_x, grid_z] = ctx.settings.space.world_grid_size;
    let chunk_size = ctx.settings.space.cell_edge_length;
    let total_w = grid_x as f32 * chunk_size;
    let total_d = grid_z as f32 * chunk_size;

    ui.horizontal(|ui| {
        let side = ui.available_height().clamp(64.0, 180.0);
        let (rect, response) = ui.allocate_exact_size(egui::vec2(side, side), Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, CornerRadius::same(4), MonuPalette::BG_DARKEST);
        painter.rect_stroke(
            rect,
            CornerRadius::same(4),
            Stroke::new(1.5, MonuPalette::ACCENT),
            egui::StrokeKind::Inside,
        );

        let inner = rect.shrink(4.0);
        let cell_w = inner.width() / grid_x.max(1) as f32;
        let cell_h = inner.height() / grid_z.max(1) as f32;
        let half_gx = (grid_x as i32) / 2;
        let half_gz = (grid_z as i32) / 2;

        for (chunk, _) in chunks_q.iter() {
            let ix = chunk.coord.x + half_gx;
            let iz = chunk.coord.y + half_gz;
            if ix >= 0 && ix < grid_x as i32 && iz >= 0 && iz < grid_z as i32 {
                let c_rect = Rect::from_min_size(
                    Pos2::new(
                        inner.left() + ix as f32 * cell_w,
                        inner.top() + iz as f32 * cell_h,
                    ),
                    egui::vec2(cell_w, cell_h),
                );
                painter.rect_filled(c_rect, 0.0, Color32::from_rgba_unmultiplied(60, 150, 85, 110));
            }
        }

        let grid_stroke = Stroke::new(0.5, Color32::from_rgba_unmultiplied(90, 100, 115, 70));
        for ix in 1..grid_x {
            let x = inner.left() + ix as f32 * cell_w;
            painter.line_segment([Pos2::new(x, inner.top()), Pos2::new(x, inner.bottom())], grid_stroke);
        }
        for iz in 1..grid_z {
            let y = inner.top() + iz as f32 * cell_h;
            painter.line_segment([Pos2::new(inner.left(), y), Pos2::new(inner.right(), y)], grid_stroke);
        }

        let world_x = ctx.cam_cell.x as f32 * chunk_size + ctx.cam_tf.translation.x;
        let world_z = ctx.cam_cell.z as f32 * chunk_size + ctx.cam_tf.translation.z;

        let norm_x = (world_x / total_w + 0.5).clamp(0.0, 1.0);
        let norm_z = (world_z / total_d + 0.5).clamp(0.0, 1.0);
        let cam_pos_2d = Pos2::new(
            inner.left() + norm_x * inner.width(),
            inner.top() + norm_z * inner.height(),
        );

        let forward = ctx.cam_tf.forward();
        let dir_2d = egui::vec2(forward.x, forward.z).normalized();
        let tip = cam_pos_2d + dir_2d * 10.0;

        painter.line_segment([cam_pos_2d, tip], Stroke::new(2.0, Color32::WHITE));
        painter.circle_filled(cam_pos_2d, 3.5, MonuPalette::ACCENT);

        if (response.clicked() || response.dragged()) && inner.width() > 1.0 {
            if let Some(pointer) = response.interact_pointer_pos() {
                let u = ((pointer.x - inner.left()) / inner.width()).clamp(0.0, 1.0);
                let v = ((pointer.y - inner.top()) / inner.height()).clamp(0.0, 1.0);

                let target_wx = (u - 0.5) * total_w;
                let target_wz = (v - 0.5) * total_d;

                let new_cx = (target_wx / chunk_size).round() as i64;
                let new_cz = (target_wz / chunk_size).round() as i64;

                ctx.cam_cell.x = new_cx;
                ctx.cam_cell.z = new_cz;
                ctx.cam_tf.translation.x = target_wx - new_cx as f32 * chunk_size;
                ctx.cam_tf.translation.z = target_wz - new_cz as f32 * chunk_size;
                ctx.cam_moved_by_ui = true;
            }
        }

        ui.vertical(|ui| {
            ui.label(RichText::new("World Radar").strong().color(MonuPalette::TEXT_PRIMARY));
            ui.label(
                RichText::new(format!("Chunks loaded: {}", chunks_q.iter().len()))
                .small()
                .color(Color32::from_rgb(100, 210, 130)),
            );
            ui.separator();
            ui.weak("ЛКМ по карте —\nбыстрый перелёт\nкамеры в точку");
        });
    });
}
