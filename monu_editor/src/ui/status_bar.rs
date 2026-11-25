use super::context::EditorUiContext;
use super::theme::MonuPalette;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, RichText, Stroke};

pub fn draw_status_bar(
    ui: &mut egui::Ui,
    ctx: &EditorUiContext,
    diagnostics: &DiagnosticsStore,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let frame_ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    egui::Frame::new()
        .fill(MonuPalette::BG_DARKEST)
        .stroke(Stroke::new(1.0, MonuPalette::BORDER))
        .inner_margin(egui::Margin::symmetric(10, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Левый блок: статус и текущий режим
                ui.label(
                    RichText::new("● Ready")
                    .small()
                    .color(Color32::from_rgb(100, 210, 130)),
                );
                ui.separator();
                ui.label(
                    RichText::new(format!("Mode: {}", ctx.state.active_mode.label()))
                    .small()
                    .color(MonuPalette::TEXT_MUTED),
                );
                ui.separator();
                ui.label(
                    RichText::new(format!(
                        "World: {:.1} km²",
                        ctx.settings.space.total_area_km2()
                    ))
                    .small()
                    .color(MonuPalette::TEXT_MUTED),
                );

                // Правый блок: FPS и время кадра
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let fps_color = if fps >= 55.0 {
                        Color32::from_rgb(100, 210, 130)
                    } else if fps >= 30.0 {
                        MonuPalette::ACCENT
                    } else {
                        Color32::from_rgb(235, 90, 90)
                    };

                    ui.label(
                        RichText::new(format!("FPS: {:.0}", fps))
                        .small()
                        .strong()
                        .color(fps_color),
                    );
                    ui.separator();
                    ui.label(
                        RichText::new(format!("Frame: {:.2} ms", frame_ms))
                        .small()
                        .color(MonuPalette::TEXT_MUTED),
                    );
                });
            });
        });
}
