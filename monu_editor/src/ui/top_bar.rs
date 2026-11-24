use super::{EditorTab, EditorUiState};
use bevy::prelude::*;
use bevy_egui::egui;
use monu_engine::camera::FlyCamera;
use monu_engine::commands::EditorCommand;
use monu_engine::CellCoord;
use monu_engine::settings::WorldSpaceSettings;

/// Главная функция отрисовки верхней панели
pub fn draw_top_bar(
    ui: &mut egui::Ui,
    ui_state: &mut EditorUiState,
    cmd_writer: &mut MessageWriter<EditorCommand>,
    space_settings: &WorldSpaceSettings,
    cam: &mut FlyCamera,
    cam_tf: &mut Transform,
    cam_cell: &mut CellCoord,
) {
    ui.horizontal(|ui| {
        ui.strong("MONU EDITOR");
        ui.separator();

        draw_add_menu(ui, cmd_writer);
        draw_panels_menu(ui, ui_state);

        if ui.button("⊞ Toggle Grid").clicked() {
            cmd_writer.write(EditorCommand::ToggleGrid);
        }

        ui.separator();
        draw_camera_speed(ui, cam);

        ui.separator();
        draw_camera_coordinates(ui, space_settings, cam_tf, cam_cell);
    });
}

/// Подметод 1: меню добавления примитивов
fn draw_add_menu(ui: &mut egui::Ui, cmd_writer: &mut MessageWriter<EditorCommand>) {
    ui.menu_button("Add", |ui| {
        if ui.button("Cube").clicked() {
            cmd_writer.write(EditorCommand::SpawnCube {
                name: "Cube".into(),
                size: Vec3::ONE,
                pos: Vec3::new(0.0, 0.5, 0.0),
            });
            ui.close();
        }
        if ui.button("Ground Plane (64m)").clicked() {
            cmd_writer.write(EditorCommand::SpawnPlane { size: 64.0 });
            ui.close();
        }
    });
}

/// Подметод 2: управление окнами и раскладкой
fn draw_panels_menu(ui: &mut egui::Ui, ui_state: &mut EditorUiState) {
    ui.menu_button("Panels", |ui| {
        for tab in EditorTab::ALL {
            let is_open = ui_state.dock_state.find_tab(&tab).is_some();
            let prefix = if is_open { "✔ " } else { "    " };
            if ui.button(format!("{prefix}{}", tab.label())).clicked() {
                ui_state.open_or_focus_tab(tab);
                ui.close();
            }
        }
        ui.separator();
        if ui.button("↺ Reset Layout").clicked() {
            ui_state.dock_state = EditorUiState::default().dock_state;
            ui.close();
        }
    });
}

/// Подметод 3: скорость камеры и пресеты (ваш вынесенный блок)
fn draw_camera_speed(ui: &mut egui::Ui, cam: &mut FlyCamera) {
    ui.label("🎥 Speed:");
    ui.add(
        egui::DragValue::new(&mut cam.speed)
        .range(0.5..=2000.0)
        .speed(0.5)
        .suffix(" m/s"),
    );

    ui.menu_button("⚙", |ui| {
        ui.strong("Camera Presets");
        ui.horizontal(|ui| {
            for (label, val) in [
                ("1 m/s", 1.0),
                ("5 m/s", 5.0),
                ("25 m/s", 25.0),
                ("100 m/s", 100.0),
                ("500 m/s", 500.0),
            ] {
                if ui.selectable_label(cam.speed == val, label).clicked() {
                    cam.speed = val;
                }
            }
        });
        ui.separator();
        ui.add(
            egui::Slider::new(&mut cam.sprint_multiplier, 1.5..=10.0)
            .text("Shift Multiplier"),
        );
        ui.add(
            egui::Slider::new(&mut cam.sensitivity, 0.0005..=0.01)
            .text("Mouse Sensitivity"),
        );
    });
}

/// Подметод 4: индикатор секции и интерактивный ввод мировых координат XYZ
fn draw_camera_coordinates(
    ui: &mut egui::Ui,
    space: &WorldSpaceSettings,
    cam_tf: &mut Transform,
    cam_cell: &mut CellCoord,
) {
    let chunk_size = space.cell_edge_length as f64;

    // 1. Индикатор секции (чанка)
    let half_chunks_x = (space.world_grid_size[0] as i64) / 2;
    let half_chunks_z = (space.world_grid_size[1] as i64) / 2;

    let in_bounds = cam_cell.x.abs() <= half_chunks_x && cam_cell.z.abs() <= half_chunks_z;

    let chunk_text = format!("📍 [{}, {}]", cam_cell.x, cam_cell.z);
    if in_bounds {
        ui.colored_label(egui::Color32::from_rgb(110, 210, 140), chunk_text);
    } else {
        ui.colored_label(
            egui::Color32::from_rgb(255, 110, 90),
            format!("{chunk_text} OUT"),
        );
    }

    // 2. Текущие мировые координаты в метрах
    let mut world_x = cam_cell.x as f64 * chunk_size + cam_tf.translation.x as f64;
    let mut world_y = cam_cell.y as f64 * chunk_size + cam_tf.translation.y as f64;
    let mut world_z = cam_cell.z as f64 * chunk_size + cam_tf.translation.z as f64;

    let resp_x = ui.add(
        egui::DragValue::new(&mut world_x)
        .speed(1.0)
        .max_decimals(1)
        .prefix("X: ")
        .suffix("m"),
    );
    let resp_y = ui.add(
        egui::DragValue::new(&mut world_y)
        .speed(1.0)
        .max_decimals(1)
        .prefix("Y: ")
        .suffix("m"),
    );
    let resp_z = ui.add(
        egui::DragValue::new(&mut world_z)
        .speed(1.0)
        .max_decimals(1)
        .prefix("Z: ")
        .suffix("m"),
    );

    // 3. Если пользователь вбил число или потянул ползунок — пересчитываем CellCoord и локальный Transform
    if resp_x.changed() || resp_y.changed() || resp_z.changed() {
        let new_cx = (world_x / chunk_size).round() as i64;
        let new_cy = (world_y / chunk_size).round() as i64;
        let new_cz = (world_z / chunk_size).round() as i64;

        cam_cell.x = new_cx;
        cam_cell.y = new_cy;
        cam_cell.z = new_cz;

        cam_tf.translation.x = (world_x - new_cx as f64 * chunk_size) as f32;
        cam_tf.translation.y = (world_y - new_cy as f64 * chunk_size) as f32;
        cam_tf.translation.z = (world_z - new_cz as f64 * chunk_size) as f32;
    }

    // Быстрый возврат в центр мира (0, 5, 12)
    if ui.button("⌖").on_hover_text("Вернуться в центр мира (0, 5, 12)").clicked() {
        *cam_cell = CellCoord::default();
        cam_tf.translation = Vec3::new(0.0, 5.0, 12.0);
    }
}
