use glam::{Vec3, Vec3Swizzles};
use noise::{NoiseFn, Perlin};
use wgpu::Device;

use crate::render::{mesh::Mesh, vertex::Vertex};

use super::create_triangle_plane;

pub struct TerrainSettings<T>
where
    T: NoiseFn<f64, 2>,
{
    pub tile_size: f32,
    pub tiles_count: u32,
    pub colors: Box<[Vec3]>,
    pub colors_thresholds: Box<[f32]>,
    pub noise: T,
    pub scale: f32,
    pub max_height: f32,
}

impl Default for TerrainSettings<Perlin> {
    fn default() -> Self {
        Self {
            tile_size: 0.75,
            tiles_count: 15,
            colors: vec![
                Vec3::new(0.94, 0.85, 0.09),
                Vec3::new(0.47, 0.83, 0.22),
                Vec3::new(0.95, 0.95, 0.95),
            ]
            .into_boxed_slice(),
            colors_thresholds: vec![-0.25, 0.5].into_boxed_slice(),
            noise: Perlin::new(Perlin::DEFAULT_SEED),
            scale: 0.2,
            max_height: 1.0,
        }
    }
}

pub fn generate_terrain_mesh<T>(device: &Device, settings: &TerrainSettings<T>) -> Mesh
where
    T: NoiseFn<f64, 2>,
{
    let mut vertices = Vec::<Vertex>::new();
    let mut indices = Vec::<u16>::new();

    fn apply_noise<T>(v: &mut Vec3, settings: &TerrainSettings<T>)
    where
        T: NoiseFn<f64, 2>,
    {
        v.y = settings
            .noise
            .get((v.xz() * settings.scale).as_dvec2().to_array()) as f32
            * settings.max_height;
    }

    fn calc_triangle_color<T>(points: [Vec3; 3], settings: &TerrainSettings<T>) -> Vec3
    where
        T: NoiseFn<f64, 2>,
    {
        let h = ((points[0] + points[1] + points[2]) / 3.0).y;
        for i in 0..settings.colors_thresholds.len() {
            if h < settings.colors_thresholds[i] {
                return settings.colors[i];
            }
        }

        *settings.colors.last().unwrap()
    }

    for x in 0..(settings.tiles_count) {
        for z in 0..(settings.tiles_count) {
            let mut v1 = Vec3::new(
                x as f32 * settings.tile_size,
                0.0,
                z as f32 * settings.tile_size,
            );
            let mut v2 = v1 + Vec3::X * settings.tile_size;
            let mut v3 = v2 + Vec3::Z * settings.tile_size;
            let mut v4 = v1 + Vec3::Z * settings.tile_size;

            apply_noise(&mut v1, settings);
            apply_noise(&mut v2, settings);
            apply_noise(&mut v3, settings);
            apply_noise(&mut v4, settings);

            let c1 = calc_triangle_color([v1, v2, v3], settings);
            vertices.extend(create_triangle_plane([v1, v2, v3], c1));
            indices.push((vertices.len() - 3) as u16);
            indices.push((vertices.len() - 2) as u16);
            indices.push((vertices.len() - 1) as u16);

            let c2 = calc_triangle_color([v1, v3, v4], settings);
            vertices.extend(create_triangle_plane([v1, v3, v4], c2));
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
