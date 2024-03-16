use std::rc::Rc;

use wgpu::{BindGroup, BindGroupLayout, Device};

pub trait BindGroupHelper {
    fn layout(&self) -> &BindGroupLayout;

    fn bind_group(&mut self, device: &Device) -> Rc<BindGroup>;
}
