use bevy::prelude::*;
use bevy_egui::egui;
use monu_engine::commands::EditorCommand;
use monu_engine::settings::LevelSettings;

pub fn draw_world_settings_tab(
    ui: &mut egui::Ui,
    settings: &mut LevelSettings,
    cmd_writer: &mut MessageWriter<EditorCommand>,
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal(|ui| {
            if ui.button("💾 Save").clicked() {
                cmd_writer.write(EditorCommand::SaveLevelSettings);
            }
            if ui.button("🔄 Reload").clicked() {
                cmd_writer.write(EditorCommand::ReloadLevelSettings);
            }
            if ui.button("Defaults").clicked() {
                *settings = LevelSettings::default();
            }
        });
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Level Name:");
            ui.text_edit_singleline(&mut settings.level_name);
        });
        ui.add_space(6.0);

        // Секция 1: BigSpace & Streaming
        egui::CollapsingHeader::new("🌍 BigSpace & Streaming")
            .default_open(true)
            .show(ui, |ui| {
                ui.add(
                    egui::Slider::new(&mut settings.space.cell_edge_length, 100.0..=8000.0)
                    .text("Chunk Size (m)"),
                );
                ui.add(
                    egui::Slider::new(&mut settings.space.switching_threshold, 50.0..=2500.0)
                    .text("Origin Threshold (m)"),
                );
                ui.add(
                    egui::Slider::new(&mut settings.space.streaming_radius, 1..=16)
                    .text("Streaming Radius (chunks)"),
                );
                ui.horizontal(|ui| {
                    ui.label("Grid Chunks (X × Z):");
                    ui.add(
                        egui::DragValue::new(&mut settings.space.world_grid_size[0])
                        .range(1..=128),
                    );
                    ui.add(
                        egui::DragValue::new(&mut settings.space.world_grid_size[1])
                        .range(1..=128),
                    );
                });

                ui.strong(format!(
                    "Total World Area: {:.1} km²",
                    settings.space.total_area_km2()
                ));
            });

        // Секция 2: Освещение и небо
        egui::CollapsingHeader::new("☀ Atmosphere & Lighting")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Sky Color:");
                    ui.color_edit_button_rgb(&mut settings.env.sky_color);
                });
                ui.add(
                    egui::Slider::new(&mut settings.env.ambient_brightness, 0.0..=2000.0)
                    .text("Ambient Light"),
                );
                ui.add(
                    egui::Slider::new(&mut settings.env.sun_pitch_deg, -90.0..=90.0)
                    .text("Sun Pitch (°)"),
                );
                ui.add(
                    egui::Slider::new(&mut settings.env.sun_yaw_deg, 0.0..=360.0)
                    .text("Sun Yaw (°)"),
                );
                ui.add(
                    egui::Slider::new(&mut settings.env.sun_illuminance, 0.0..=100_000.0)
                    .logarithmic(true)
                    .text("Sun Lux"),
                );
            });

        // Секция 3: Туман и физика
        egui::CollapsingHeader::new("🌫 Fog & Physics")
            .default_open(true)
            .show(ui, |ui| {
                ui.checkbox(&mut settings.env.fog_enabled, "Enable Distance Fog");
                ui.add_enabled(
                    settings.env.fog_enabled,
                    egui::Slider::new(&mut settings.env.fog_start, 10.0..=10_000.0)
                    .text("Fog Start (m)"),
                );
                ui.add_enabled(
                    settings.env.fog_enabled,
                    egui::Slider::new(&mut settings.env.fog_end, 50.0..=20_000.0)
                    .text("Fog End (m)"),
                );
                ui.separator();
                ui.add(
                    egui::DragValue::new(&mut settings.env.gravity)
                    .speed(0.1)
                    .prefix("Gravity (m/s²): "),
                );
            });
    });
}
