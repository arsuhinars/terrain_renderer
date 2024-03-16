use std::cell::RefCell;

use wgpu::{BindGroup, CommandEncoder, Queue, TextureView};

use super::scene::Camera;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderStage {
    OPAQUE,
    TRANSPARENT,
}

pub struct RenderingContext<'a> {
    camera: &'a RefCell<Camera>,
    surface_view: &'a TextureView,
    depth_view: &'a TextureView,
    scene_bind_group: &'a BindGroup,
    queue: &'a RefCell<Queue>,
    encoder: &'a RefCell<Option<CommandEncoder>>,
}

impl<'a> RenderingContext<'a> {
    pub fn new(
        camera: &'a RefCell<Camera>,
        surface_view: &'a TextureView,
        depth_view: &'a TextureView,
        scene_bind_group: &'a BindGroup,
        queue: &'a RefCell<Queue>,
        encoder: &'a RefCell<Option<CommandEncoder>>,
    ) -> RenderingContext<'a> {
        RenderingContext {
            camera,
            surface_view,
            depth_view,
            scene_bind_group,
            queue,
            encoder,
        }
    }

    pub fn camera(&self) -> &RefCell<Camera> {
        &self.camera
    }

    pub fn surface_view(&self) -> &TextureView {
        &self.surface_view
    }

    pub fn depth_view(&self) -> &TextureView {
        &self.depth_view
    }

    pub fn scene_bind_group(&self) -> &BindGroup {
        &self.scene_bind_group
    }

    pub fn queue(&self) -> &RefCell<Queue> {
        self.queue
    }

    pub fn encoder(&self) -> &RefCell<Option<CommandEncoder>> {
        self.encoder
    }
}

pub trait Renderer {
    fn render(&mut self, context: &RenderingContext);

    fn stage(&self) -> RenderStage;
}
