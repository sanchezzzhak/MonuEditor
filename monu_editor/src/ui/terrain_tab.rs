use super::context::EditorUiContext;
use super::theme::MonuPalette;
use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, CornerRadius, RichText, Stroke};
use monu_engine::terrain::{TerrainBrushMode, TerrainCommand};

pub fn draw_terrain_tab(ui: &mut egui::Ui, ctx: &mut EditorUiContext) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label(
            RichText::new("TERRAIN TOOLS")
            .strong()
            .size(12.0)
            .color(MonuPalette::TEXT_MUTED),
        );
        ui.separator();

        // 1. Управление чанками и сохранением
        let cur_coord = IVec2::new(ctx.cam_cell.x as i32, ctx.cam_cell.z as i32);
        if ui
            .add_sized(
                [ui.available_width(), 26.0],
                egui::Button::new(format!("➕ Create Chunk [{}, {}]", cur_coord.x, cur_coord.y)),
            )
            .clicked()
        {
            ctx.terrain_cmd.write(TerrainCommand::CreateChunk {
                coord: cur_coord,
                resolution: 129,
            });
        }

        ui.horizontal(|ui| {
            let half_w = (ui.available_width() - 6.0) * 0.5;
            if ui.add_sized([half_w, 24.0], egui::Button::new("💾 Save All")).clicked() {
                ctx.terrain_cmd.write(TerrainCommand::SaveAllChunks);
            }
            if ui.add_sized([half_w, 24.0], egui::Button::new("📂 Load All")).clicked() {
                ctx.terrain_cmd.write(TerrainCommand::LoadAllChunks);
            }
        });

        if !ctx.has_terrain {
            ui.add_space(8.0);
            ui.weak("Создайте чанк в текущей секции или загрузите сохранённый мир.");
            return;
        }

        ui.horizontal(|ui| {
            let w = (ui.available_width() - 12.0) / 3.0;
            if ui.add_sized([w, 24.0], egui::Button::new("⛰ Hills")).clicked() {
                ctx.terrain_cmd.write(TerrainCommand::GenerateHills {
                    seed: 42,
                    amplitude: 85.0,
                });
            }
            if ui.add_sized([w, 24.0], egui::Button::new("🧵 Stitch")).clicked() {
                ctx.terrain_cmd.write(TerrainCommand::StitchAllSeams);
            }
            if ui.add_sized([w, 24.0], egui::Button::new("Flat")).clicked() {
                ctx.terrain_cmd.write(TerrainCommand::ResetFlat);
            }
        });

        ui.add_space(6.0);
        ui.separator();

        // 2. Включение кисти + крупная сетка инструментов 2x2
        ui.checkbox(&mut ctx.brush.enabled, "🖌 Enable Sculpt Brush (ЛКМ)");
        ui.add_space(4.0);

        ui.add_enabled_ui(ctx.brush.enabled, |ui| {
            let modes = [
                (TerrainBrushMode::Raise, "⬆ Raise\nПоднять"),
                (TerrainBrushMode::Lower, "⬇ Lower\nОпустить"),
                (TerrainBrushMode::Smooth, "〰 Smooth\nСгладить"),
                (TerrainBrushMode::Flatten, "═ Flatten\nВыровнять"),
            ];

            egui::Grid::new("brush_modes_grid")
                .num_columns(2)
                .spacing([6.0, 6.0])
                .show(ui, |ui| {
                    let btn_w = (ui.available_width() - 6.0) * 0.5;
                    for (idx, (mode, label)) in modes.iter().enumerate() {
                        let is_active = ctx.brush.mode == *mode;
                        let btn = if is_active {
                            egui::Button::new(RichText::new(*label).strong().color(Color32::WHITE))
                            .fill(MonuPalette::BG_WIDGET_HOVER)
                            .stroke(Stroke::new(1.5, MonuPalette::ACCENT))
                            .corner_radius(CornerRadius::same(5))
                        } else {
                            egui::Button::new(RichText::new(*label).color(MonuPalette::TEXT_MUTED))
                            .fill(MonuPalette::BG_DARKEST)
                            .stroke(Stroke::new(1.0, MonuPalette::BORDER))
                            .corner_radius(CornerRadius::same(5))
                        };

                        if ui.add_sized([btn_w, 42.0], btn).clicked() {
                            ctx.brush.mode = *mode;
                        }
                        if idx % 2 == 1 {
                            ui.end_row();
                        }
                    }
                });

            ui.add_space(8.0);
            ui.label(
                RichText::new("BRUSH SETTINGS")
                .strong()
                .size(11.0)
                .color(MonuPalette::TEXT_MUTED),
            );
            ui.add(egui::Slider::new(&mut ctx.brush.radius, 2.0..=300.0).text("Radius (m)"));
            ui.add(egui::Slider::new(&mut ctx.brush.strength, 1.0..=200.0).text("Strength"));
            ui.add(egui::Slider::new(&mut ctx.brush.falloff, 0.2..=4.0).text("Falloff"));

            if matches!(ctx.brush.mode, TerrainBrushMode::Flatten) {
                ui.add(
                    egui::DragValue::new(&mut ctx.brush.target_height)
                    .speed(0.5)
                    .prefix("Target Height (m): "),
                );
            }
        });
    });
}
