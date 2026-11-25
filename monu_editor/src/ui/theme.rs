use bevy_egui::egui::{self, Color32, CornerRadius, Stroke, Visuals};
use egui_dock::Style as DockStyle;

/// Фирменная палитра Monu Editor (в стиле UE5 Dark + Amber Accent)
pub struct MonuPalette;

impl MonuPalette {
    pub const BG_DARKEST: Color32 = Color32::from_rgb(18, 20, 24);      // Фон табов и полос
    pub const BG_PANEL: Color32 = Color32::from_rgb(26, 29, 35);        // Основной фон панелей
    pub const BG_WIDGET: Color32 = Color32::from_rgb(38, 42, 50);       // Кнопки, поля ввода
    pub const BG_WIDGET_HOVER: Color32 = Color32::from_rgb(50, 56, 67); // Наведение на кнопку
    pub const BORDER: Color32 = Color32::from_rgb(48, 53, 63);          // Тонкие разделители
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(225, 229, 236); // Основной текст
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(138, 145, 158);   // Второстепенный текст
    pub const ACCENT: Color32 = Color32::from_rgb(224, 138, 30);        // Оранжевый акцент UE5
    pub const ACCENT_FILL: Color32 = Color32::from_rgb(190, 112, 20);   // Заливка активных кнопок
}

/// Применяет тёмную тему ко всему egui Context
pub fn apply_monu_theme(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();

    // Компактные, аккуратные отступы как в профессиональных DCC/движках
    style.spacing.item_spacing = egui::vec2(6.0, 5.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    style.spacing.window_margin = egui::Margin::same(8);
    style.spacing.slider_width = 130.0;

    let mut visuals = Visuals::dark();

    // Фоны окон и панелей
    visuals.window_fill = MonuPalette::BG_PANEL;
    visuals.panel_fill = MonuPalette::BG_PANEL;
    visuals.faint_bg_color = MonuPalette::BG_DARKEST;
    visuals.extreme_bg_color = MonuPalette::BG_DARKEST;

    // Обводки и разделители
    visuals.window_stroke = Stroke::new(1.0, MonuPalette::BORDER);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, MonuPalette::BORDER);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, MonuPalette::TEXT_PRIMARY);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(4);

    // Обычные кнопки и поля (Inactive)
    visuals.widgets.inactive.bg_fill = MonuPalette::BG_WIDGET;
    visuals.widgets.inactive.weak_bg_fill = MonuPalette::BG_WIDGET;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, MonuPalette::BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, MonuPalette::TEXT_PRIMARY);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(4);

    // Наведение мыши (Hovered)
    visuals.widgets.hovered.bg_fill = MonuPalette::BG_WIDGET_HOVER;
    visuals.widgets.hovered.weak_bg_fill = MonuPalette::BG_WIDGET_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, MonuPalette::ACCENT);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(4);

    // Нажатие / Активный виджет (Active)
    visuals.widgets.active.bg_fill = MonuPalette::ACCENT_FILL;
    visuals.widgets.active.weak_bg_fill = MonuPalette::ACCENT_FILL;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, MonuPalette::ACCENT);
    visuals.widgets.active.fg_stroke = Stroke::new(1.5, Color32::WHITE);
    visuals.widgets.active.corner_radius = CornerRadius::same(4);

    // Открытые выпадающие меню (Open)
    visuals.widgets.open.bg_fill = MonuPalette::BG_WIDGET_HOVER;
    visuals.widgets.open.weak_bg_fill = MonuPalette::BG_WIDGET_HOVER;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, MonuPalette::ACCENT);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.open.corner_radius = CornerRadius::same(4);

    // Выделение текста и активных элементов (Selection)
    visuals.selection.bg_fill = MonuPalette::ACCENT_FILL;
    visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);

    style.visuals = visuals;
    ctx.set_global_style(style);
}

/// Настраивает внешний вид вкладок и разделителей egui_dock под палитру Monu Editor
pub fn build_dock_style(ctx: &egui::Context) -> DockStyle {
    let mut dock_style = DockStyle::from_egui(ctx.global_style().as_ref());

    // Полоса заголовков вкладок
    dock_style.tab_bar.bg_fill = MonuPalette::BG_DARKEST;
    dock_style.tab_bar.height = 26.0;
    dock_style.tab_bar.hline_color = MonuPalette::BORDER;

    // Активная вкладка с оранжевой верхней полоской
    dock_style.tab.active.bg_fill = MonuPalette::BG_PANEL;
    dock_style.tab.active.text_color = Color32::WHITE;
    dock_style.tab.active.outline_color = MonuPalette::BORDER;

    // Неактивные вкладки
    dock_style.tab.inactive.bg_fill = MonuPalette::BG_DARKEST;
    dock_style.tab.inactive.text_color = MonuPalette::TEXT_MUTED;
    dock_style.tab.inactive.outline_color = Color32::TRANSPARENT;

    // Вкладка при наведении
    dock_style.tab.hovered.bg_fill = MonuPalette::BG_WIDGET;
    dock_style.tab.hovered.text_color = Color32::WHITE;

    // Границы и сплиттеры между панелями
    dock_style.separator.width = 2.0;
    dock_style.separator.color_idle = MonuPalette::BG_DARKEST;
    dock_style.separator.color_hovered = MonuPalette::ACCENT;
    dock_style.separator.color_dragged = MonuPalette::ACCENT;

    // Убираем внешние отступы вокруг всей сетки панелей
    dock_style.main_surface_border_stroke = Stroke::new(1.0, MonuPalette::BORDER);
    dock_style.separator.width = 1.0; // Тонкий стык в 1px вместо толстых щелей
    dock_style.separator.color_idle = MonuPalette::BORDER;
    dock_style.separator.color_hovered = MonuPalette::ACCENT;
    dock_style.separator.color_dragged = MonuPalette::ACCENT;

    // Убираем внутренние отступы внутри тела вкладки (мы сами делаем аккуратные карточки)
    dock_style.tab.tab_body.inner_margin = egui::Margin::same(6);
    dock_style.tab.tab_body.stroke = Stroke::NONE;

    dock_style
}
