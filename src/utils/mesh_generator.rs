use glam::Vec3;
use wgpu::Device;

use crate::render::{mesh::Mesh, vertex::Vertex};

use super::create_triangle_plane;

pub struct TerrainSettings {
    pub tile_size: f32,
    pub tiles_count: u32,
    pub corner_position: Vec3,
    pub color: Vec3,
}

impl Default for TerrainSettings {
    fn default() -> Self {
        Self {
            tile_size: 1.0,
            tiles_count: 10,
            corner_position: Vec3::ZERO,
            color: Vec3::new(0.47, 0.83, 0.22),
        }
    }
}

pub fn generate_terrain_mesh(device: &Device, settings: &TerrainSettings) -> Mesh {
    let mut vertices = Vec::<Vertex>::new();
    let mut indices = Vec::<u16>::new();

    for x in 0..(settings.tiles_count) {
        for z in 0..(settings.tiles_count) {
            let v1 = Vec3::new(
                x as f32 * settings.tile_size,
                0.0,
                z as f32 * settings.tile_size,
            );
            let v2 = v1 + Vec3::X * settings.tile_size;
            let v3 = v2 + Vec3::Z * settings.tile_size;
            let v4 = v1 + Vec3::Z * settings.tile_size;

            vertices.extend(create_triangle_plane([v1, v2, v3], settings.color));
            indices.push((vertices.len() - 3) as u16);
            indices.push((vertices.len() - 2) as u16);
            indices.push((vertices.len() - 1) as u16);

            vertices.extend(create_triangle_plane([v1, v3, v4], settings.color));
            indices.push((vertices.len() - 3) as u16);
            indices.push((vertices.len() - 2) as u16);
            indices.push((vertices.len() - 1) as u16);
        }
    }

    Mesh::new(
        device,
        vertices.into_boxed_slice(),
        indices.into_boxed_slice(),
    )
}
