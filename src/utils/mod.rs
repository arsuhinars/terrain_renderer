use bytemuck::{bytes_of, Pod};
use glam::Vec3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType,
    BufferUsages, Device, Extent3d, ImageCopyTexture, Origin3d, ShaderStages, Texture,
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use crate::render::{renderer::RenderingContext, vertex::Vertex};

pub mod terrain_generator;

pub fn create_texture_2d(
    device: &Device,
    format: TextureFormat,
    width: u32,
    height: u32,
    usage: TextureUsages,
) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: None,
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

pub fn copy_textures_2d(context: &RenderingContext, source: &Texture, target: &Texture) {
    context
        .encoder()
        .borrow_mut()
        .as_mut()
        .unwrap()
        .copy_texture_to_texture(
            ImageCopyTexture {
                texture: source,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            ImageCopyTexture {
                texture: target,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            Extent3d {
                width: source.width(),
                height: source.height(),
                depth_or_array_layers: 1,
            },
        );
}

pub fn create_uniform_init(
    uniform: &impl Pod,
    device: &Device,
) -> (Buffer, BindGroupLayout, BindGroup) {
    let buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytes_of(uniform),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::all(),
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::Buffer(BufferBinding {
                buffer: &buffer,
                offset: 0,
                size: None,
            }),
        }],
    });

    (buffer, bind_group_layout, bind_group)
}

pub fn create_triangle_plane(points: [Vec3; 3], color: Vec3) -> [Vertex; 3] {
    let a = points[1] - points[0];
    let b = points[2] - points[0];
    let n = a.cross(b);

    [
        Vertex::new(points[0], n, color),
        Vertex::new(points[1], n, color),
        Vertex::new(points[2], n, color),
    ]
}
