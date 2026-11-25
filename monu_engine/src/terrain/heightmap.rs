use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Сетка высот для одного чанка ландшафта
#[derive(Component, Reflect, Serialize, Deserialize, Debug, Clone)]
#[reflect(Component)]
pub struct TerrainHeightmap {
    /// Количество вершин по стороне (например, 65 или 129)
    pub resolution: usize,
    /// Физический размер стороны чанка в метрах (берётся из LevelSettings)
    pub size_meters: f32,
    /// Максимальная амплитуда высоты в метрах
    pub max_height: f32,
    /// Одномерный массив высот [z * resolution + x] в метрах
    pub heights: Vec<f32>,
}

impl TerrainHeightmap {
    pub fn new_flat(resolution: usize, size_meters: f32, max_height: f32) -> Self {
        let res = resolution.max(2);
        Self {
            resolution: res,
            size_meters,
            max_height,
            heights: vec![0.0; res * res],
        }
    }

    #[inline]
    pub fn idx(&self, x: usize, z: usize) -> usize {
        z * self.resolution + x
    }

    pub fn get(&self, x: usize, z: usize) -> f32 {
        self.heights[self.idx(x, z)]
    }

    pub fn set(&mut self, x: usize, z: usize, val: f32) {
        let i = self.idx(x, z);
        self.heights[i] = val.clamp(-self.max_height, self.max_height);
    }

    /// Перевод индексов сетки (x, z) в локальные координаты (от -size/2 до +size/2)
    pub fn grid_to_local(&self, x: usize, z: usize) -> Vec2 {
        let step = self.size_meters / (self.resolution - 1) as f32;
        let half = self.size_meters * 0.5;
        Vec2::new(-half + x as f32 * step, -half + z as f32 * step)
    }

    /// Быстрый поиск точки пересечения луча с поверхностью ландшафта (в локальных координатах чанка)
    pub fn raycast_local(&self, ray_origin: Vec3, ray_dir: Vec3) -> Option<Vec3> {
        let step = self.size_meters / (self.resolution - 1) as f32;
        let half = self.size_meters * 0.5;
        let max_dist = self.size_meters * 2.5;
        let march_step = (step * 0.5).max(0.25);

        let mut t = 0.0;
        let mut prev_pos = ray_origin;
        let mut prev_diff = ray_origin.y - self.sample_height_local(ray_origin.x, ray_origin.z, half, step);

        while t <= max_dist {
            t += march_step;
            let pos = ray_origin + ray_dir * t;
            let h = self.sample_height_local(pos.x, pos.z, half, step);
            let diff = pos.y - h;

            if prev_diff >= 0.0 && diff <= 0.0 {
                // Уточнение бисекцией (5 итераций)
                let mut a = prev_pos;
                let mut b = pos;
                for _ in 0..5 {
                    let mid = (a + b) * 0.5;
                    let mh = self.sample_height_local(mid.x, mid.z, half, step);
                    if mid.y - mh >= 0.0 {
                        a = mid;
                    } else {
                        b = mid;
                    }
                }
                let hit = (a + b) * 0.5;
                if hit.x.abs() <= half && hit.z.abs() <= half {
                    return Some(hit);
                }
            }
            prev_pos = pos;
            prev_diff = diff;
        }
        None
    }

    fn sample_height_local(&self, lx: f32, lz: f32, half: f32, step: f32) -> f32 {
        let fx = ((lx + half) / step).clamp(0.0, (self.resolution - 1) as f32);
        let fz = ((lz + half) / step).clamp(0.0, (self.resolution - 1) as f32);
        let x0 = fx.floor() as usize;
        let z0 = fz.floor() as usize;
        let x1 = (x0 + 1).min(self.resolution - 1);
        let z1 = (z0 + 1).min(self.resolution - 1);
        let tx = fx - x0 as f32;
        let tz = fz - z0 as f32;

        let h00 = self.get(x0, z0);
        let h10 = self.get(x1, z0);
        let h01 = self.get(x0, z1);
        let h11 = self.get(x1, z1);

        let h0 = h00 + (h10 - h00) * tx;
        let h1 = h01 + (h11 - h01) * tx;
        h0 + (h1 - h0) * tz
    }

    /// Генерация Bevy Mesh с расчётом нормалей и автоматической окраской по высоте и склону
    pub fn build_mesh(&self) -> Mesh {
        let res = self.resolution;
        let vert_count = res * res;
        let step = self.size_meters / (res - 1) as f32;
        let half = self.size_meters * 0.5;

        let mut positions = Vec::with_capacity(vert_count);
        let mut normals = Vec::with_capacity(vert_count);
        let mut uvs = Vec::with_capacity(vert_count);
        let mut colors = Vec::with_capacity(vert_count);

        for z in 0..res {
            for x in 0..res {
                let px = -half + x as f32 * step;
                let pz = -half + z as f32 * step;
                let py = self.get(x, z);
                positions.push([px, py, pz]);
                uvs.push([x as f32 / (res - 1) as f32, z as f32 / (res - 1) as f32]);

                // Нормаль по центральным разностям
                let hl = self.get(x.saturating_sub(1), z);
                let hr = self.get((x + 1).min(res - 1), z);
                let hd = self.get(x, z.saturating_sub(1));
                let hu = self.get(x, (z + 1).min(res - 1));
                let n = Vec3::new(hl - hr, 2.0 * step, hd - hu).normalize_or_zero();
                normals.push(n.to_array());

                // Процедурный цвет по высоте и крутизне склона (песок -> трава -> скалы -> снег)
                let slope = 1.0 - n.y; // 0 = плоско, 1 = отвесная стена
                let color = if py < -1.5 {
                    [0.76, 0.70, 0.50, 1.0] // Песок / дно
                } else if slope > 0.35 {
                    [0.42, 0.40, 0.38, 1.0] // Скалы на крутых склонах
                } else if py > self.max_height * 0.65 {
                    [0.92, 0.94, 0.96, 1.0] // Снежные вершины
                } else {
                    [0.24, 0.48, 0.22, 1.0] // Трава / луга
                };
                colors.push(color);
            }
        }

        let mut indices = Vec::with_capacity((res - 1) * (res - 1) * 6);
        for z in 0..(res - 1) {
            for x in 0..(res - 1) {
                let i0 = (z * res + x) as u32;
                let i1 = i0 + 1;
                let i2 = ((z + 1) * res + x) as u32;
                let i3 = i2 + 1;

                indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
            }
        }

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_indices(Indices::U32(indices));
        mesh
    }
}

