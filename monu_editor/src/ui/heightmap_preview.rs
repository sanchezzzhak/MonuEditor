use super::context::EditorUiContext;
use super::theme::MonuPalette;
use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, ColorImage, CornerRadius, RichText, Stroke, TextureHandle, TextureOptions};
use monu_engine::terrain::{TerrainChunk, TerrainHeightmap};

/// Ресурс кэша текстуры превью карты высот (чтобы не пересоздавать каждый кадр)
#[derive(Resource, Default)]
pub struct HeightmapPreviewState {
    pub texture: Option<TextureHandle>,
    pub cached_coord: Option<IVec2>,
    pub cached_hash: u64,
    pub min_h: f32,
    pub max_h: f32,
    pub avg_h: f32,
}

pub fn draw_heightmap_preview_tab(
    ui: &mut egui::Ui,
    ctx: &EditorUiContext,
    chunks_q: &Query<(&TerrainChunk, &TerrainHeightmap)>,
    preview_state: &mut HeightmapPreviewState,
) {
    let target_coord = IVec2::new(ctx.cam_cell.x as i32, ctx.cam_cell.z as i32);

    // Ищем чанк под камерой, либо берём первый существующий чанк
    let selected_chunk = chunks_q
        .iter()
        .find(|(c, _)| c.coord == target_coord)
        .or_else(|| chunks_q.iter().next());

    let Some((chunk, hm)) = selected_chunk else {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.weak("Нет активного чанка ландшафта");
            ui.weak("Создайте чанк во вкладке Terrain Tools");
        });
        return;
    };

    // Быстрый хэш высот, чтобы перерисовывать картинку только при реальном изменении рельефа
    let current_hash = compute_heightmap_hash(&hm.heights);
    if preview_state.texture.is_none()
        || preview_state.cached_coord != Some(chunk.coord)
        || preview_state.cached_hash != current_hash
    {
        let (img, min_h, max_h, avg_h) = build_topographic_image(hm);
        preview_state.min_h = min_h;
        preview_state.max_h = max_h;
        preview_state.avg_h = avg_h;
        preview_state.cached_coord = Some(chunk.coord);
        preview_state.cached_hash = current_hash;

        if let Some(tex) = &mut preview_state.texture {
            tex.set(img, TextureOptions::LINEAR);
        } else {
            preview_state.texture = Some(ui.ctx().load_texture(
                "heightmap_preview_tex",
                img,
                TextureOptions::LINEAR,
            ));
        }
    }

    ui.horizontal(|ui| {
        // Левая колонка: квадратное превью карты высот
        if let Some(tex) = &preview_state.texture {
            let side = ui.available_height().clamp(64.0, 180.0);
            egui::Frame::new()
                .stroke(Stroke::new(1.0, MonuPalette::BORDER))
                .corner_radius(CornerRadius::same(4))
                .show(ui, |ui| {
                    ui.image(egui::load::SizedTexture::new(
                        tex.id(),
                        egui::vec2(side, side),
                    ));
                });
        }

        // Правая колонка: телеметрия высот
        ui.vertical(|ui| {
            ui.label(
                RichText::new(format!("Chunk [{}, {}]", chunk.coord.x, chunk.coord.y))
                .strong()
                .color(MonuPalette::ACCENT),
            );
            ui.label(
                RichText::new(format!("Grid: {}×{}", hm.resolution, hm.resolution))
                .small()
                .color(MonuPalette::TEXT_MUTED),
            );
            ui.separator();

            ui.monospace(format!("Max: {:+.1} m", preview_state.max_h));
            ui.monospace(format!("Avg: {:+.1} m", preview_state.avg_h));
            ui.monospace(format!("Min: {:+.1} m", preview_state.min_h));
        });
    });
}

fn compute_heightmap_hash(heights: &[f32]) -> u64 {
    // Быстрое сэмплирование каждой 4-й вершины для моментальной проверки изменений
    let mut hash: u64 = 1469598103934665603;
    for &h in heights.iter().step_by(4) {
        hash ^= h.to_bits() as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

fn build_topographic_image(hm: &TerrainHeightmap) -> (ColorImage, f32, f32, f32) {
    let res = hm.resolution;
    let mut pixels = Vec::with_capacity(res * res);

    let mut min_h = f32::MAX;
    let mut max_h = f32::MIN;
    let mut sum_h = 0.0f64;

    for &h in &hm.heights {
        if h < min_h {
            min_h = h;
        }
        if h > max_h {
            max_h = h;
        }
        sum_h += h as f64;
    }
    let avg_h = if hm.heights.is_empty() {
        0.0
    } else {
        (sum_h / hm.heights.len() as f64) as f32
    };

    let span = (max_h - min_h).max(1.0);

    for &h in &hm.heights {
        let t = ((h - min_h) / span).clamp(0.0, 1.0);
        // Гипсометрическая раскраска (глубокий синий -> зелёный -> охра -> скалы -> белый снег)
        let color = if h < -1.0 {
            Color32::from_rgb(38, 86, 145)
        } else if t < 0.35 {
            lerp_color(
                Color32::from_rgb(52, 118, 64),
                Color32::from_rgb(104, 158, 78),
                t / 0.35,
            )
        } else if t < 0.72 {
            lerp_color(
                Color32::from_rgb(104, 158, 78),
                Color32::from_rgb(138, 122, 102),
                (t - 0.35) / 0.37,
            )
        } else {
            lerp_color(
                Color32::from_rgb(138, 122, 102),
                Color32::from_rgb(240, 244, 248),
                (t - 0.72) / 0.28,
            )
        };
        pixels.push(color);
    }

    (
        ColorImage::new([res, res], pixels),
        min_h,
        max_h,
        avg_h,
    )
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
    )
}
