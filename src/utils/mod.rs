use bytemuck::{bytes_of, Pod};
use glam::Vec3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType,
    BufferUsages, Device, ShaderStages,
};

use crate::render::vertex::Vertex;

pub mod mesh_generator;

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

fn create_triangle_plane(points: [Vec3; 3], color: Vec3) -> [Vertex; 3] {
    let a = points[1] - points[0];
    let b = points[2] - points[0];
    let n = a.cross(b);

    [
        Vertex::new(points[0], n, color),
        Vertex::new(points[1], n, color),
        Vertex::new(points[2], n, color),
    ]
}
