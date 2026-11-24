use bevy::prelude::*;
use bevy_egui::egui;
use monu_engine::commands::EditorCommand;
use monu_engine::world::LevelObject;

pub fn draw_details_tab(
    ui: &mut egui::Ui,
    selected: &mut Option<Entity>,
    cmd_writer: &mut MessageWriter<EditorCommand>,
    objects_q: &mut Query<(Entity, &Name, &mut Transform), With<LevelObject>>,
) {
    ui.heading("Transform");
    ui.separator();
    if let Some(sel) = *selected {
        if let Ok((_, name, mut tf)) = objects_q.get_mut(sel) {
            ui.label(format!("Selected: {}", name.as_str()));
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("Location:");
                ui.add(egui::DragValue::new(&mut tf.translation.x).speed(0.1).prefix("X: "));
                ui.add(egui::DragValue::new(&mut tf.translation.y).speed(0.1).prefix("Y: "));
                ui.add(egui::DragValue::new(&mut tf.translation.z).speed(0.1).prefix("Z: "));
            });

            ui.horizontal(|ui| {
                ui.label("Scale:    ");
                ui.add(egui::DragValue::new(&mut tf.scale.x).speed(0.05).prefix("X: "));
                ui.add(egui::DragValue::new(&mut tf.scale.y).speed(0.05).prefix("Y: "));
                ui.add(egui::DragValue::new(&mut tf.scale.z).speed(0.05).prefix("Z: "));
            });

            ui.add_space(12.0);
            if ui.button("🗑 Delete Object").clicked() {
                cmd_writer.write(EditorCommand::DeleteEntity(sel));
                *selected = None;
            }
        } else {
            *selected = None;
        }
    } else {
        ui.weak("Выберите объект в Outliner");
    }
}
