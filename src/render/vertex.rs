use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use wgpu::{vertex_attr_array, VertexAttribute, VertexBufferLayout, VertexStepMode};

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec3,
}

static VERTEX_ATTRIBUTES: [VertexAttribute; 3] = vertex_attr_array![
    0 => Float32x3,
    1 => Float32x3,
    2 => Float32x3
];

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3, color: Vec3) -> Vertex {
        Vertex {
            position,
            normal,
            color,
        }
    }

    pub fn attributes() -> &'static [VertexAttribute] {
        &VERTEX_ATTRIBUTES
    }

    pub fn buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Vertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &VERTEX_ATTRIBUTES,
        }
    }
}
