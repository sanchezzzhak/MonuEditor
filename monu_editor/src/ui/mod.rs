mod details;
mod outliner;
mod world_settings;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use egui_dock::{DockArea, DockState, Style, TabViewer};
use monu_engine::commands::EditorCommand;
use monu_engine::settings::LevelSettings;
use monu_engine::world::LevelObject;

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<EditorUiState>()
            .add_systems(EguiPrimaryContextPass, draw_editor_ui);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EditorTab {
    Viewport,
    Outliner,
    Details,
    WorldSettings,
    ContentBrowser,
}

#[derive(Resource)]
pub struct EditorUiState {
    pub dock_state: DockState<EditorTab>,
    pub selected: Option<Entity>,
    pub pointer_over_ui: bool,
}

impl Default for EditorUiState {
    fn default() -> Self {
        let mut dock_state = DockState::new(vec![EditorTab::Viewport]);
        let surface = dock_state.main_surface_mut();

        let [center, _right] = surface.split_right(
            egui_dock::NodeIndex::root(),
            0.22,
            vec![EditorTab::Details, EditorTab::WorldSettings],
        );
        let [center, _left] = surface.split_left(center, 0.20, vec![EditorTab::Outliner]);
        let [_center, _bottom] =
            surface.split_below(center, 0.24, vec![EditorTab::ContentBrowser]);

        Self {
            dock_state,
            selected: None,
            pointer_over_ui: false,
        }
    }
}

fn draw_editor_ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<EditorUiState>,
    mut cmd_writer: MessageWriter<EditorCommand>,
    mut level_settings: ResMut<LevelSettings>,
    mut objects_q: Query<(Entity, &Name, &mut Transform), With<LevelObject>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let dock_style = Style::from_egui(ctx.global_style().as_ref());

    egui::Area::new(egui::Id::new("editor_root_area"))
        .fixed_pos(egui::Pos2::ZERO)
        .default_size(ctx.content_rect().size())
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            ui.set_min_size(ctx.content_rect().size());

            egui::Panel::top("top_menu_bar").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.strong("MONU EDITOR");
                    ui.separator();

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

                    if ui.button("Toggle Grid").clicked() {
                        cmd_writer.write(EditorCommand::ToggleGrid);
                    }
                });
            });

            let mut dock_state =
                std::mem::replace(&mut ui_state.dock_state, DockState::new(vec![]));
            let mut viewer = MonuTabViewer {
                selected: &mut ui_state.selected,
                cmd_writer: &mut cmd_writer,
                level_settings: &mut level_settings,
                objects_q: &mut objects_q,
            };

            DockArea::new(&mut dock_state)
                .style(dock_style)
                .show_inside(ui, &mut viewer);

            ui_state.dock_state = dock_state;
        });

    ui_state.pointer_over_ui = ctx.is_pointer_over_egui();
}

struct MonuTabViewer<'a, 'w1, 'w2, 's, 'n, 't> {
    selected: &'a mut Option<Entity>,
    cmd_writer: &'a mut MessageWriter<'w1, EditorCommand>,
    level_settings: &'a mut LevelSettings,
    objects_q: &'a mut Query<'w2, 's, (Entity, &'n Name, &'t mut Transform), With<LevelObject>>,
}

impl TabViewer for MonuTabViewer<'_, '_, '_, '_, '_, '_> {
    type Tab = EditorTab;

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(tab)
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            EditorTab::Viewport => "Viewport".into(),
            EditorTab::Outliner => "Outliner".into(),
            EditorTab::Details => "Details".into(),
            EditorTab::WorldSettings => "World Settings".into(),
            EditorTab::ContentBrowser => "Content / Terrain".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            EditorTab::Viewport => {}
            EditorTab::Outliner => {
                outliner::draw_outliner_tab(ui, self.selected, self.objects_q);
            }
            EditorTab::Details => {
                details::draw_details_tab(ui, self.selected, self.cmd_writer, self.objects_q);
            }
            EditorTab::WorldSettings => {
                world_settings::draw_world_settings_tab(ui, self.level_settings, self.cmd_writer);
            }
            EditorTab::ContentBrowser => {
                ui.label("Здесь разместим модуль генератора 2D-карты и ландшафта.");
            }
        }
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, EditorTab::Viewport)
    }
}
