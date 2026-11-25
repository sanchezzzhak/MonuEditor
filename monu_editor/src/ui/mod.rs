mod details;
mod outliner;
mod world_settings;
mod top_bar;
mod terrain_tab;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use egui_dock::{DockArea, DockState, Style, TabViewer};
use monu_engine::CellCoord;
use monu_engine::terrain::{TerrainBrush, TerrainChunk, TerrainCommand};
use monu_engine::camera::FlyCamera;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorTab {
    Viewport,
    Outliner,
    Details,
    WorldSettings,
    Terrain,
    ContentBrowser,

}

impl EditorTab {
    const ALL: [EditorTab; 6] = [
        EditorTab::Viewport,
        EditorTab::Outliner,
        EditorTab::Details,
        EditorTab::WorldSettings,
        EditorTab::Terrain,
        EditorTab::ContentBrowser,
    ];

    fn label(self) -> &'static str {
        match self {
            EditorTab::Viewport => "Viewport",
            EditorTab::Outliner => "Outliner",
            EditorTab::Details => "Details",
            EditorTab::Terrain => "Terrain",
            EditorTab::WorldSettings => "World Settings",
            EditorTab::ContentBrowser => "Content / Terrain",
        }
    }
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

        // Правая панель: Details, WorldSettings и Terrain
        let [center, _right] = surface.split_right(
            egui_dock::NodeIndex::root(),
            0.78,
            vec![
                EditorTab::Details,
                EditorTab::WorldSettings,
                EditorTab::Terrain,
            ],
        );

        // Левая панель: Outliner
        let [center, _left] = surface.split_left(center, 0.20, vec![EditorTab::Outliner]);

        // Нижняя панель: ContentBrowser
        let [_center, _bottom] =
            surface.split_below(center, 0.76, vec![EditorTab::ContentBrowser]);

        Self {
            dock_state,
            selected: None,
            pointer_over_ui: false,
        }
    }
}

impl EditorUiState {
    /// Открывает вкладку заново, если её закрыли, или переключает на неё фокус
    pub fn open_or_focus_tab(&mut self, tab: EditorTab) {
        if let Some(location) = self.dock_state.find_tab(&tab) {
            let _ = self.dock_state.set_active_tab(location);
        } else {
            self.dock_state.push_to_first_leaf(tab);
        }
    }
}

fn draw_editor_ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<EditorUiState>,
    mut cmd_writer: MessageWriter<EditorCommand>,
    mut terrain_cmd: MessageWriter<TerrainCommand>,
    mut level_settings: ResMut<LevelSettings>,
    mut terrain_brush: ResMut<TerrainBrush>,
    mut objects_q: Query<(Entity, &Name, &mut Transform), With<LevelObject>>,
    mut cam_q: Query<(&mut FlyCamera, &mut Transform, &mut CellCoord), Without<LevelObject>>,
    terrain_q: Query<(), With<TerrainChunk>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let dock_style = Style::from_egui(ctx.global_style().as_ref());

    egui::Area::new(egui::Id::new("editor_root_area"))
        .fixed_pos(egui::Pos2::ZERO)
        .default_size(ctx.content_rect().size())
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            ui.set_min_size(ctx.content_rect().size());
            
            // Верхняя панель меню (вынесена в top_bar.rs)
            egui::Panel::top("top_menu_bar").show(ui, |ui| {
                if let Ok((mut cam, mut cam_tf, mut cam_cell)) = cam_q.single_mut() {
                    let space_snapshot = level_settings.space.clone();
                    top_bar::draw_top_bar(
                        ui,
                        &mut ui_state,
                        &mut cmd_writer,
                        &space_snapshot,
                        &mut cam,
                        &mut cam_tf,
                        &mut cam_cell,
                    );
                }
            });


            // Рабочее пространство с докингом вкладок
            let mut dock_state =
                std::mem::replace(&mut ui_state.dock_state, DockState::new(vec![]));
            let has_terrain = !terrain_q.is_empty();

            let mut viewer = MonuTabViewer {
                selected: &mut ui_state.selected,
                cmd_writer: &mut cmd_writer,
                terrain_cmd: &mut terrain_cmd,
                level_settings: &mut level_settings,
                terrain_brush: &mut terrain_brush,
                objects_q: &mut objects_q,
                has_terrain,
            };

            DockArea::new(&mut dock_state)
                .style(dock_style)
                .show_inside(ui, &mut viewer);

            ui_state.dock_state = dock_state;
        });

    let over_ui = ctx.wants_pointer_input();
    ui_state.pointer_over_ui = over_ui;
    terrain_brush.pointer_over_ui = over_ui;
    
}

struct MonuTabViewer<'a, 'w1, 'w2, 'w3, 's, 'n, 't> {
    selected: &'a mut Option<Entity>,
    cmd_writer: &'a mut MessageWriter<'w1, EditorCommand>,
    terrain_cmd: &'a mut MessageWriter<'w2, TerrainCommand>,
    level_settings: &'a mut LevelSettings,
    terrain_brush: &'a mut TerrainBrush,
    objects_q: &'a mut Query<'w3, 's, (Entity, &'n Name, &'t mut Transform), With<LevelObject>>,
    has_terrain: bool,
}

impl TabViewer for MonuTabViewer<'_, '_, '_, '_, '_, '_, '_> {
    type Tab = EditorTab;

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(tab)
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.label().into()
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        // Главный Viewport нельзя случайно закрыть крестиком
        !matches!(tab, EditorTab::Viewport)
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
            EditorTab::Terrain => {
                terrain_tab::draw_terrain_tab(
                    ui,
                    self.terrain_brush,
                    self.terrain_cmd,
                    self.has_terrain,
                );
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
