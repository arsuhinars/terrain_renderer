use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

use crate::render::vertex::Vertex;

pub struct Mesh {
    vertices: Box<[Vertex]>,
    indices: Box<[u16]>,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl Mesh {
    pub fn new(device: &Device, vertices: Box<[Vertex]>, indices: Box<[u16]>) -> Mesh {
        let vertex_buffer = Self::create_vertex_buffer(device, &vertices);
        let index_buffer = Self::create_index_buffer(device, &indices);

        Mesh {
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn from_slices(device: &Device, vertices: &[Vertex], indices: &[u16]) -> Mesh {
        let mut vertices_vec = Vec::<Vertex>::new();
        let mut indices_vec = Vec::<u16>::new();

        vertices_vec.extend_from_slice(vertices);
        indices_vec.extend_from_slice(indices);

        Self::new(
            device,
            vertices_vec.into_boxed_slice(),
            indices_vec.into_boxed_slice(),
        )
    }

    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    pub fn indices(&self) -> &[u16] {
        &self.indices
    }

    pub fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    fn create_vertex_buffer(device: &Device, vertices: &[Vertex]) -> Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(vertices),
            usage: BufferUsages::VERTEX,
        })
    }

    fn create_index_buffer(device: &Device, indices: &[u16]) -> Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        })
    }
}
