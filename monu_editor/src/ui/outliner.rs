use bevy::prelude::*;
use bevy_egui::egui;
use monu_engine::world::LevelObject;

pub fn draw_outliner_tab(
    ui: &mut egui::Ui,
    selected: &mut Option<Entity>,
    objects_q: &Query<(Entity, &Name, &mut Transform), With<LevelObject>>,
) {
    ui.heading("Scene Objects");
    ui.separator();
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (entity, name, _) in objects_q.iter() {
            let is_selected = *selected == Some(entity);
            if ui
                .selectable_label(is_selected, format!("📦 {}", name.as_str()))
                .clicked()
            {
                *selected = Some(entity);
            }
        }
    });
}
