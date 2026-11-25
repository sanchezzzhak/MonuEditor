use super::theme::MonuPalette;
use bevy_egui::egui::{self, Image, ImageSource};

pub struct EditorIcons;

impl EditorIcons {
    pub const SIZE_NORM: f32 = 16.0;

    // pub const MOUNTAIN: ImageSource<'static> =
    //     egui::include_image!("../../assets/icons/mountain.svg");
    // pub const ARROW_UP: ImageSource<'static> =
    //     egui::include_image!("../../assets/icons/arrow-up.svg");
    // pub const ARROW_DOWN: ImageSource<'static> =
    //     egui::include_image!("../../assets/icons/arrow-down.svg");
    // pub const SMOOTH: ImageSource<'static> =
    //     egui::include_image!("../../assets/icons/wave-sine.svg");

    pub const SAVE: ImageSource<'static> =
        egui::include_image!("../../assets/icons/device-floppy.svg");
    // pub const LOAD: ImageSource<'static> =
    //     egui::include_image!("../../assets/icons/folder-open.svg");
}

/// Удобный хелпер для отрисовки SVG-иконки нужного размера и цвета
pub fn svg_icon(source: ImageSource<'static>, size_px: f32, active: bool) -> Image<'static> {
    let tint = if active {
        MonuPalette::ACCENT
    } else {
        MonuPalette::TEXT_PRIMARY
    };

    Image::new(source)
    .fit_to_exact_size(egui::vec2(size_px, size_px))
    .tint(tint)
}
