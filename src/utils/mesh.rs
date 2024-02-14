use crate::render::vertex::Vertex;

pub struct Mesh {
    vertices: Box<[Vertex]>,
    indices: Box<[u16]>,
}
