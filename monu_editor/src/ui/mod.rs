mod context;
mod icons;
mod details;
mod outliner;
mod status_bar;
mod terrain_tab;
mod theme;
mod top_bar;
mod world_settings;
mod heightmap_preview;
mod minimap;

use heightmap_preview::HeightmapPreviewState;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use context::EditorUiContext;
use egui_dock::{DockArea, DockState, TabViewer};
use monu_engine::camera::FlyCamera;
use monu_engine::commands::EditorCommand;
use monu_engine::settings::LevelSettings;
use monu_engine::terrain::{TerrainBrush, TerrainChunk, TerrainCommand, TerrainHeightmap};
use monu_engine::world::LevelObject;
use monu_engine::CellCoord;


pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EguiPlugin::default(), FrameTimeDiagnosticsPlugin::default()))
            .init_resource::<EditorUiState>()
            .init_resource::<HeightmapPreviewState>()
            .add_systems(EguiPrimaryContextPass, draw_editor_ui);

    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EditorMode {
    #[default]
    Terrain,
    Blueprint,
    Materials,
    Animation,
    Vfx,
    Plugins,
}

impl EditorMode {
    pub const ALL: [EditorMode; 6] = [
        EditorMode::Terrain,
        EditorMode::Blueprint,
        EditorMode::Materials,
        EditorMode::Animation,
        EditorMode::Vfx,
        EditorMode::Plugins,
    ];

    pub fn label(self) -> &'static str {
        match self {
            EditorMode::Terrain => "🏔 Terrain",
            EditorMode::Blueprint => "🕸 Blueprint",
            EditorMode::Materials => "🎨 Materials",
            EditorMode::Animation => "🎬 Animation",
            EditorMode::Vfx => "✨ VFX",
            EditorMode::Plugins => "🧩 Plugins",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorTab {
    Viewport,
    Tools,
    Outliner,
    Details,
    WorldSettings,
    ContentBrowser,
    MaskEditor,
    HeightmapPreview,
    Minimap,
}

impl EditorTab {
    const ALL: [EditorTab; 9] = [
        EditorTab::Viewport,
        EditorTab::Tools,
        EditorTab::Outliner,
        EditorTab::Details,
        EditorTab::WorldSettings,
        EditorTab::ContentBrowser,
        EditorTab::MaskEditor,
        EditorTab::HeightmapPreview,
        EditorTab::Minimap,
    ];


    fn label(self) -> &'static str {
        match self {
            EditorTab::Viewport => "Viewport",
            EditorTab::Tools => "Terrain Tools",
            EditorTab::Outliner => "World Outliner",
            EditorTab::Details => "Details",
            EditorTab::WorldSettings => "World Settings",
            EditorTab::ContentBrowser => "Content / Textures",
            EditorTab::MaskEditor => "Mask Editor",
            EditorTab::HeightmapPreview => "Heightmap Preview",
            EditorTab::Minimap => "Navigation",

        }
    }
}

#[derive(Resource)]
pub struct EditorUiState {
    pub dock_state: DockState<EditorTab>,
    pub selected: Option<Entity>,
    pub pointer_over_ui: bool,
    pub active_mode: EditorMode,
}

impl Default for EditorUiState {
    fn default() -> Self {
        let mut dock_state = DockState::new(vec![EditorTab::Viewport]);
        let surface = dock_state.main_surface_mut();

        // 1. Нижняя полоса на всю ширину (24% высоты снизу)
        let [top_area, bottom_1] = surface.split_below(
            egui_dock::NodeIndex::root(),
            0.76,
            vec![EditorTab::ContentBrowser],
        );
        
        // Делим нижнюю полосу на 4 равные колонки
        let [_bottom_1, bottom_2] = surface.split_right(bottom_1, 0.25, vec![EditorTab::MaskEditor]);
        let [_bottom_2, bottom_3] = surface.split_right(bottom_2, 0.33, vec![EditorTab::HeightmapPreview]);
        let [_bottom_3, _bottom_4] = surface.split_right(bottom_3, 0.50, vec![EditorTab::Minimap]);


        // 2. Правая колонка в верхней части (20% ширины)
        let [center, right] = surface.split_right(top_area, 0.80, vec![EditorTab::Outliner]);

        // Делим правую колонку по вертикали: сверху Outliner (40%), снизу Details + World Settings (60%)
        let [_right_top, _right_bottom] = surface.split_below(
            right,
            0.40,
            vec![EditorTab::Details, EditorTab::WorldSettings],
        );

        // 3. Левая колонка инструментов (18% ширины)
        let [_viewport, _left] = surface.split_left(center, 0.22, vec![EditorTab::Tools]);

        Self {
            dock_state,
            selected: None,
            pointer_over_ui: false,
            active_mode: EditorMode::Terrain,
        }
    }
}

impl EditorUiState {
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
    mut preview_state: ResMut<HeightmapPreviewState>,
    mut objects_q: Query<(Entity, &Name, &mut Transform), With<LevelObject>>,
    mut cam_q: Query<(&mut FlyCamera, &mut Transform, &mut CellCoord), Without<LevelObject>>,
    chunks_q: Query<(&TerrainChunk, &TerrainHeightmap)>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui_extras::install_image_loaders(&*ctx);
    theme::apply_monu_theme(ctx);
    let dock_style = theme::build_dock_style(ctx);

    let mut dock_state = std::mem::replace(&mut ui_state.dock_state, DockState::new(vec![]));
    let (init_tf, init_cell) = cam_q
        .single()
        .map(|(_, tf, c)| (*tf, *c))
        .unwrap_or_default();

    let mut editor_ctx = EditorUiContext {
        state: &mut ui_state,
        cmd: &mut cmd_writer,
        terrain_cmd: &mut terrain_cmd,
        settings: &mut level_settings,
        brush: &mut terrain_brush,
        objects: &mut objects_q,
        has_terrain: !chunks_q.is_empty(),
        cam_cell: init_cell,
        cam_tf: init_tf,
        cam_moved_by_ui: false,
    };

    egui::Area::new(egui::Id::new("editor_root_area"))
        .fixed_pos(egui::Pos2::ZERO)
        .default_size(ctx.content_rect().size())
        .order(egui::Order::Background)
        .interactable(true)
        .show(ctx, |ui| {
            ui.set_min_size(ctx.content_rect().size());

            let zero_frame = egui::Frame::new()
                .inner_margin(egui::Margin::ZERO)
                .outer_margin(egui::Margin::ZERO);

            // Верхняя двухэтажная шапка
            egui::Panel::top("top_header_container")
                .frame(zero_frame)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                    if let Ok((mut cam, mut cam_tf, mut cam_cell)) = cam_q.single_mut() {
                        top_bar::draw_top_bar(
                            ui,
                            &mut editor_ctx,
                            &mut cam,
                            &mut cam_tf,
                            &mut cam_cell,
                        );
                    }
                });

            // Нижняя строка состояния (FPS / Frame ms / Status)
            egui::Panel::bottom("bottom_status_bar")
                .frame(zero_frame)
                .show(ui, |ui| {
                    status_bar::draw_status_bar(ui, &editor_ctx, &diagnostics);
                });
            
            // Рабочее пространство с докингом вкладок
            let mut viewer = MonuTabViewer {
                ctx: &mut editor_ctx,
                chunks_q: &chunks_q,
                preview_state: &mut preview_state,
            };

            DockArea::new(&mut dock_state)
                .style(dock_style)
                .show_leaf_collapse_buttons(false)
                .show_leaf_close_all_buttons(false)
                .show_inside(ui, &mut viewer);
        });

    if editor_ctx.cam_moved_by_ui {
        if let Ok((_, mut cam_tf, mut cam_cell)) = cam_q.single_mut() {
            *cam_tf = editor_ctx.cam_tf;
            *cam_cell = editor_ctx.cam_cell;
        }
    }

    ui_state.dock_state = dock_state;

    let over_ui = ctx.egui_is_using_pointer();
    ui_state.pointer_over_ui = over_ui;
    terrain_brush.pointer_over_ui = over_ui;
}

struct MonuTabViewer<'a, 'b, 'w1, 'w2, 'w3, 'w4, 's1, 's2, 'n, 't, 'c, 'h> {
    ctx: &'a mut EditorUiContext<'b, 'w1, 'w2, 'w3, 's1, 'n, 't>,
    chunks_q: &'a Query<'w4, 's2, (&'c TerrainChunk, &'h TerrainHeightmap)>,
    preview_state: &'a mut HeightmapPreviewState,
}


impl TabViewer for MonuTabViewer<'_, '_, '_, '_, '_, '_, '_, '_, '_, '_, '_, '_> {
    type Tab = EditorTab;

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(tab)
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            EditorTab::Tools => match self.ctx.state.active_mode {
                EditorMode::Terrain => "Terrain Tools".into(),
                EditorMode::Blueprint => "Blueprint Tools".into(),
                EditorMode::Materials => "Material Tools".into(),
                EditorMode::Animation => "Animation Tools".into(),
                EditorMode::Vfx => "VFX Tools".into(),
                EditorMode::Plugins => "Plugins".into(),
            },
            _ => tab.label().into(),
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        !matches!(tab, EditorTab::Viewport)
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            EditorTab::Viewport => {}
            EditorTab::Tools => match self.ctx.state.active_mode {
                EditorMode::Terrain => terrain_tab::draw_terrain_tab(ui, self.ctx),
                other => {
                    ui.heading(other.label());
                    ui.separator();
                    ui.weak("Инструменты этого режима будут добавлены на следующих этапах.");
                }
            },
            EditorTab::Outliner => {
                outliner::draw_outliner_tab(ui, &mut self.ctx.state.selected, self.ctx.objects);
            }
            EditorTab::Details => {
                details::draw_details_tab(
                    ui,
                    &mut self.ctx.state.selected,
                    self.ctx.cmd,
                    self.ctx.objects,
                );
            }
            EditorTab::WorldSettings => {
                world_settings::draw_world_settings_tab(ui, self.ctx.settings, self.ctx.cmd);
            }
            EditorTab::ContentBrowser => {
                ui.weak("Библиотека текстур и материалов ландшафта.");
            }
            EditorTab::MaskEditor => {
                ui.weak("Редактор масок биомов (трава, скалы, снег, дороги).");
            }
            EditorTab::HeightmapPreview => {
                heightmap_preview::draw_heightmap_preview_tab(
                    ui,
                    self.ctx,
                    self.chunks_q,
                    self.preview_state,
                );
            }
            EditorTab::Minimap => {
                minimap::draw_minimap_tab(ui, self.ctx, self.chunks_q);
            }
        }
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, EditorTab::Viewport)
    }
}
