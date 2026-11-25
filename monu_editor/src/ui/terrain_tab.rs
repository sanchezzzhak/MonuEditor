use bevy::prelude::*;
use bevy_egui::egui;
use monu_engine::terrain::{TerrainBrush, TerrainBrushMode, TerrainCommand};

pub fn draw_terrain_tab(
    ui: &mut egui::Ui,
    brush: &mut TerrainBrush,
    terrain_cmd: &mut MessageWriter<TerrainCommand>,
    has_terrain: bool,
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("🏔 Landscape / Terrain");
        ui.separator();

        // 1. Создание / генерация чанка
        ui.horizontal(|ui| {
            if ui.button("➕ Create Terrain (129×129)").clicked() {
                terrain_cmd.write(TerrainCommand::CreateChunk {
                    coord: IVec2::ZERO,
                    resolution: 129,
                });
            }
            if has_terrain {
                if ui.button("⛰ Generate Hills").clicked() {
                    terrain_cmd.write(TerrainCommand::GenerateHills {
                        seed: 42,
                        amplitude: 85.0,
                    });
                }
                if ui.button("Flat Reset").clicked() {
                    terrain_cmd.write(TerrainCommand::ResetFlat);
                }
            }
        });

        ui.add_space(8.0);
        if !has_terrain {
            ui.weak("Нажмите «Create Terrain», чтобы создать ландшафт в текущей секции [0, 0].");
            return;
        }

        // 2. Включение кисти скульптинга
        ui.checkbox(&mut brush.enabled, "🖌 Enable Sculpt Brush (ЛКМ по земле)");
        ui.separator();

        ui.add_enabled_ui(brush.enabled, |ui| {
            ui.label("Brush Mode:");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut brush.mode, TerrainBrushMode::Raise, "⬆ Raise");
                ui.selectable_value(&mut brush.mode, TerrainBrushMode::Lower, "⬇ Lower");
                ui.selectable_value(&mut brush.mode, TerrainBrushMode::Smooth, "〰 Smooth");
                ui.selectable_value(&mut brush.mode, TerrainBrushMode::Flatten, "═ Flatten");
            });

            ui.add_space(6.0);
            ui.add(
                egui::Slider::new(&mut brush.radius, 2.0..=250.0)
                .text("Brush Radius (m)"),
            );
            ui.add(
                egui::Slider::new(&mut brush.strength, 1.0..=150.0)
                .text("Strength (m/s)"),
            );
            ui.add(
                egui::Slider::new(&mut brush.falloff, 0.2..=4.0)
                .text("Falloff"),
            );

            if matches!(brush.mode, TerrainBrushMode::Flatten) {
                ui.add(
                    egui::DragValue::new(&mut brush.target_height)
                    .speed(0.5)
                    .prefix("Target Height (m): "),
                );
            }
        });
    });
}
