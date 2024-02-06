use wgpu::{BindGroup, CommandEncoder, Queue, TextureView};

use super::scene::Camera;

pub trait Renderer {
    fn render(
        &mut self,
        camera: &mut Camera,
        surface_view: &TextureView,
        depth_view: &TextureView,
        scene_bind_group: &BindGroup,
        queue: &mut Queue,
        encoder: &mut CommandEncoder,
    );
}
