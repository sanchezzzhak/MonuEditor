use super::context::EditorUiContext;
use super::theme::MonuPalette;
use super::{EditorMode, EditorTab, EditorUiState};
use super::icons::{svg_icon, EditorIcons};

use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, CornerRadius, RichText, Stroke};
use monu_engine::camera::FlyCamera;
use monu_engine::commands::EditorCommand;
use monu_engine::settings::WorldSpaceSettings;
use monu_engine::CellCoord;

/// Главная функция отрисовки двухэтажной шапки в стиле UE5
pub fn draw_top_bar(
    ui: &mut egui::Ui,
    ctx: &mut EditorUiContext,
    cam: &mut FlyCamera,
    cam_tf: &mut Transform,
    cam_cell: &mut CellCoord,
) {
    // ЭТАЖ 1: Верхняя строка меню (Логотип MONU | File Edit Window Tools Help | Имя карты)
    egui::Frame::new()
        .fill(MonuPalette::BG_DARKEST)
        .stroke(Stroke::new(1.0, MonuPalette::BORDER))
        .inner_margin(egui::Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("▲ MONU")
                    .strong()
                    .size(14.0)
                    .color(MonuPalette::ACCENT),
                );
                ui.separator();

                draw_main_menus(ui, ctx.state, ctx.cmd);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("● Engine Ready")
                        .small()
                        .color(Color32::from_rgb(100, 210, 130)),
                    );
                    ui.separator();

                    egui::Frame::new()
                        .fill(MonuPalette::BG_WIDGET)
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(egui::Margin::symmetric(8, 2))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(format!("🗺 {}", ctx.settings.level_name))
                                .small()
                                .strong()
                                .color(MonuPalette::TEXT_PRIMARY),
                            );
                        });
                });
            });
        });

    // ЭТАЖ 2: Основной тулбар (Быстрые действия | Переключатель режимов | Камера и Координаты)
    egui::Frame::new()
        .fill(MonuPalette::BG_PANEL)
        .stroke(Stroke::new(1.0, MonuPalette::BORDER))
        .inner_margin(egui::Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .button("💾")
                    .on_hover_text("Сохранить настройки уровня")
                    .clicked()
                {
                    ctx.cmd.write(EditorCommand::SaveLevelSettings);
                }

                draw_add_menu(ui, ctx.cmd);

                if ui.button("⊞ Grid").clicked() {
                    ctx.cmd.write(EditorCommand::ToggleGrid);
                }

                ui.separator();
                draw_mode_switcher(ui, ctx.state);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    draw_camera_coordinates(ui, &ctx.settings.space, cam_tf, cam_cell);
                    ui.separator();
                    draw_camera_speed(ui, cam);
                });
            });
        });
}

/// Подметод: классическое верхнее меню File / Edit / Window / Tools / Help
fn draw_main_menus(
    ui: &mut egui::Ui,
    ui_state: &mut EditorUiState,
    cmd_writer: &mut MessageWriter<EditorCommand>,
) {
    ui.menu_button("File", |ui| {

        if ui.add(egui::Button::image_and_text(
            svg_icon(EditorIcons::SAVE, EditorIcons::SIZE_NORM, false),
            "Save All",
        )).clicked() {
            cmd_writer.write(EditorCommand::SaveLevelSettings);
            ui.close();
        }

        if ui.button("💾 Save Level Settings").clicked() {
            cmd_writer.write(EditorCommand::SaveLevelSettings);
            ui.close();
        }
        if ui.button("🔄 Reload Level Settings").clicked() {
            cmd_writer.write(EditorCommand::ReloadLevelSettings);
            ui.close();
        }
    });

    ui.menu_button("Edit", |ui| {
        if ui.button("🌍 World Settings").clicked() {
            ui_state.open_or_focus_tab(EditorTab::WorldSettings);
            ui.close();
        }
    });

    ui.menu_button("Window", |ui| {
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

    ui.menu_button("Tools", |ui| {
        if ui.button("🏔 Landscape Sculpting").clicked() {
            ui_state.active_mode = EditorMode::Terrain;
            ui_state.open_or_focus_tab(EditorTab::Tools);
            ui.close();
        }
        ui.weak("🗺 2D Map Generator (в разработке)");
        ui.weak("🌿 Foliage Scatter (в разработке)");
    });

    ui.menu_button("Help", |ui| {
        ui.strong("Управление камерой:");
        ui.label("ПКМ + WASD — полёт");
        ui.label("Shift — ускорение");
        ui.label("Space / Ctrl — вверх / вниз");
    });
}

/// Подметод: кнопки-вкладки выбора режима редактора
fn draw_mode_switcher(ui: &mut egui::Ui, ui_state: &mut EditorUiState) {
    ui.horizontal(|ui| {
        for mode in EditorMode::ALL {
            let is_active = ui_state.active_mode == mode;

            let btn = if is_active {
                egui::Button::new(RichText::new(mode.label()).strong().color(Color32::WHITE))
                .fill(MonuPalette::BG_WIDGET_HOVER)
                .stroke(Stroke::new(1.5, MonuPalette::ACCENT))
                .corner_radius(CornerRadius::same(5))
            } else {
                egui::Button::new(RichText::new(mode.label()).color(MonuPalette::TEXT_MUTED))
                .fill(MonuPalette::BG_DARKEST)
                .stroke(Stroke::new(1.0, MonuPalette::BORDER))
                .corner_radius(CornerRadius::same(5))
            };

            if ui.add_sized([92.0, 24.0], btn).clicked() {
                ui_state.active_mode = mode;
                if mode == EditorMode::Terrain {
                    ui_state.open_or_focus_tab(EditorTab::Tools);
                }
            }
        }
    });
}

/// Подметод: меню добавления объектов
fn draw_add_menu(ui: &mut egui::Ui, cmd_writer: &mut MessageWriter<EditorCommand>) {
    ui.menu_button("➕ Add", |ui| {
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

/// Подметод: скорость камеры и пресеты
fn draw_camera_speed(ui: &mut egui::Ui, cam: &mut FlyCamera) {
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

    ui.add(
        egui::DragValue::new(&mut cam.speed)
        .range(0.5..=2000.0)
        .speed(0.5)
        .suffix(" m/s"),
    );
    ui.label("🎥");
}

/// Подметод: индикатор секции и интерактивный ввод мировых координат XYZ
fn draw_camera_coordinates(
    ui: &mut egui::Ui,
    space: &WorldSpaceSettings,
    cam_tf: &mut Transform,
    cam_cell: &mut CellCoord,
) {
    let chunk_size = space.cell_edge_length as f64;

    if ui
        .button("⌖")
        .on_hover_text("Вернуться в центр мира (0, 5, 12)")
        .clicked()
    {
        *cam_cell = CellCoord::default();
        cam_tf.translation = Vec3::new(0.0, 5.0, 12.0);
    }

    let mut world_x = cam_cell.x as f64 * chunk_size + cam_tf.translation.x as f64;
    let mut world_y = cam_cell.y as f64 * chunk_size + cam_tf.translation.y as f64;
    let mut world_z = cam_cell.z as f64 * chunk_size + cam_tf.translation.z as f64;

    let resp_z = ui.add(
        egui::DragValue::new(&mut world_z)
        .speed(1.0)
        .max_decimals(1)
        .prefix("Z: ")
        .suffix("m"),
    );
    let resp_y = ui.add(
        egui::DragValue::new(&mut world_y)
        .speed(1.0)
        .max_decimals(1)
        .prefix("Y: ")
        .suffix("m"),
    );
    let resp_x = ui.add(
        egui::DragValue::new(&mut world_x)
        .speed(1.0)
        .max_decimals(1)
        .prefix("X: ")
        .suffix("m"),
    );

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

    let half_chunks_x = (space.world_grid_size[0] as i64) / 2;
    let half_chunks_z = (space.world_grid_size[1] as i64) / 2;
    let in_bounds = cam_cell.x.abs() <= half_chunks_x && cam_cell.z.abs() <= half_chunks_z;
    let chunk_text = format!("📍 [{}, {}]", cam_cell.x, cam_cell.z);
    if in_bounds {
        ui.colored_label(Color32::from_rgb(110, 210, 140), chunk_text);
    } else {
        ui.colored_label(
            Color32::from_rgb(255, 110, 90),
            format!("{chunk_text} OUT"),
        );
    }
}
