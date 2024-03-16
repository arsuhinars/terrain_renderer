use std::{cell::RefCell, collections::HashMap, iter, sync::Arc};

use glam::{Quat, Vec2, Vec3};
use wgpu::{
    Adapter, Color, Device, DeviceDescriptor, Instance, Operations, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RequestAdapterOptions, Surface, SurfaceConfiguration, Texture, TextureFormat, TextureUsages,
    TextureView,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    core::time_manager::TimeManager,
    utils::{copy_textures_2d, create_texture_2d},
};

use super::{
    bind_group::BindGroupHelper,
    renderer::{RenderStage, Renderer, RenderingContext},
    scene::{Camera, SceneBindGroup},
};

#[derive(Clone, Copy)]
pub struct RenderSettings {
    clear_color: Color,

    camera_fov: f32,
    camera_near_plane: f32,
    camera_far_plane: f32,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            clear_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            camera_fov: 60.0,
            camera_near_plane: 0.1,
            camera_far_plane: 100.0,
        }
    }
}

pub struct RenderManager<'a> {
    settings: Box<RenderSettings>,
    surface_config: wgpu::SurfaceConfiguration,
    surface: Surface<'a>,
    device: Device,
    queue: RefCell<Queue>,
    depth_texture: Texture,
    depth_view: TextureView,

    camera: Box<RefCell<Camera>>,

    scene_bind_group: Box<RefCell<SceneBindGroup>>,

    renderers_by_stage: HashMap<RenderStage, Vec<Box<dyn Renderer>>>,
}

impl<'a> RenderManager<'a> {
    pub async fn new(
        settings: &RenderSettings,
        window: Arc<Window>,
    ) -> Result<RenderManager<'a>, String> {
        let instance: Instance = Instance::new(Default::default());

        let (surface_width, surface_height) = window.inner_size().into();
        let surface = instance
            .create_surface(window.clone())
            .map_err(|err| err.to_string())?;

        let (adapter, device, queue) = Self::create_wgpu_objects(&instance, &surface).await?;

        let surface_config =
            Self::create_surface_config(&surface, &adapter, surface_width, surface_height);

        surface.configure(&device, &surface_config);

        let depth_texture = create_texture_2d(
            &device,
            TextureFormat::Depth32Float,
            surface_width,
            surface_height,
            TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        );
        let depth_view = depth_texture.create_view(&Default::default());

        let opaque_texture = create_texture_2d(
            &device,
            surface_config.format,
            surface_width,
            surface_height,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        );
        let opaque_depth_texture = create_texture_2d(
            &device,
            TextureFormat::Depth32Float,
            surface_width,
            surface_height,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        );

        let camera = Camera::new(
            Vec3::ZERO,
            Quat::IDENTITY,
            settings.camera_fov,
            (surface_width as f32) / (surface_height as f32),
            settings.camera_near_plane,
            settings.camera_far_plane,
        );

        let scene_bind_group = SceneBindGroup::new(&device, opaque_texture, opaque_depth_texture);

        Ok(RenderManager {
            settings: Box::new(*settings),
            surface_config,
            surface,
            device,
            queue: RefCell::new(queue),
            depth_texture,
            depth_view,

            camera: Box::new(RefCell::new(camera)),

            scene_bind_group: Box::new(RefCell::new(scene_bind_group)),

            renderers_by_stage: HashMap::from([
                (RenderStage::OPAQUE, Vec::new()),
                (RenderStage::TRANSPARENT, Vec::new()),
            ]),
        })
    }

    pub fn add_renderer(&mut self, renderer: Box<dyn Renderer>) {
        let v = self.renderers_by_stage.get(&renderer.stage());
        if v.is_none() {
            self.renderers_by_stage.insert(renderer.stage(), Vec::new());
        }

        self.renderers_by_stage
            .get_mut(&renderer.stage())
            .unwrap()
            .push(renderer);
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.surface_config.format
    }

    pub fn depth_texture(&self) -> &Texture {
        &self.depth_texture
    }

    pub fn scene_bind_group(&self) -> &RefCell<SceneBindGroup> {
        self.scene_bind_group.as_ref()
    }

    pub fn camera(&self) -> &RefCell<Camera> {
        &self.camera
    }

    pub fn handle_resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);

        let mut scene_bind_group = self.scene_bind_group.borrow_mut();

        let mut uniform = *scene_bind_group.uniform();
        uniform.surface_size = Vec2::new(size.width as f32, size.height as f32);
        scene_bind_group.update_uniform(&self.queue.borrow(), &uniform);

        self.depth_texture = create_texture_2d(
            &self.device,
            self.depth_texture.format(),
            size.width,
            size.height,
            self.depth_texture.usage(),
        );
        self.depth_view = self.depth_texture.create_view(&Default::default());

        let opaque_texture = create_texture_2d(
            &self.device,
            self.surface_format(),
            size.width,
            size.height,
            scene_bind_group.opaque_texture().usage(),
        );

        let opaque_depth_texture = create_texture_2d(
            &self.device,
            self.depth_texture.format(),
            size.width,
            size.height,
            scene_bind_group.opaque_depth_texture().usage(),
        );

        scene_bind_group.update_textures(opaque_texture, opaque_depth_texture);

        self.camera
            .borrow_mut()
            .set_aspect_ratio((size.width as f32) / (size.height as f32));
    }

    pub fn render(&mut self, time_manager: &TimeManager) -> Result<(), String> {
        let surface = self
            .surface
            .get_current_texture()
            .map_err(|err| err.to_string())?;
        let surface_view = surface.texture.create_view(&Default::default());

        let mut scene_bind_group = self.scene_bind_group.borrow_mut();

        let encoder = RefCell::new(Some(
            self.device.create_command_encoder(&Default::default()),
        ));

        {
            let mut camera_ref = self.camera.borrow_mut();
            let mut uniform = *scene_bind_group.uniform();

            uniform.view_proj_matrix = camera_ref.view_proj_matrix();
            uniform.camera_dir = camera_ref.look_dir();
            uniform.camera_pos = camera_ref.position();
            uniform.camera_near = camera_ref.near_plane();
            uniform.camera_far = camera_ref.far_plane();
            uniform.time += time_manager.delta();

            scene_bind_group.update_uniform(&self.queue.borrow(), &uniform);
        }

        let wgpu_bind_group = scene_bind_group.bind_group(&self.device);

        let mut context = RenderingContext::new(
            &self.camera,
            &surface_view,
            &self.depth_view,
            wgpu_bind_group.as_ref(),
            &self.queue,
            &encoder,
        );

        self.clear_surface(&context);

        for renderer in self
            .renderers_by_stage
            .get_mut(&RenderStage::OPAQUE)
            .unwrap()
        {
            renderer.render(&mut context);
        }

        copy_textures_2d(
            &context,
            &surface.texture,
            scene_bind_group.opaque_texture(),
        );
        copy_textures_2d(
            &context,
            &self.depth_texture,
            scene_bind_group.opaque_depth_texture(),
        );

        for renderer in self
            .renderers_by_stage
            .get_mut(&RenderStage::TRANSPARENT)
            .unwrap()
        {
            renderer.render(&mut context);
        }

        self.queue
            .borrow()
            .submit(iter::once(encoder.replace(None).unwrap().finish()));

        surface.present();

        Ok(())
    }

    async fn create_wgpu_objects(
        instance: &Instance,
        surface: &Surface<'a>,
    ) -> Result<(Adapter, Device, Queue), String> {
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .ok_or("Requested adapter was None")?;

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    ..Default::default()
                },
                None,
            )
            .await
            .map_err(|err| err.to_string())?;

        Ok((adapter, device, queue))
    }

    fn create_surface_config(
        surface: &Surface,
        adapter: &Adapter,
        width: u32,
        height: u32,
    ) -> SurfaceConfiguration {
        let surface_capabilities = surface.get_capabilities(adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_capabilities.formats[0]);
        let present_mode = surface_capabilities
            .present_modes
            .iter()
            .copied()
            .filter(|m| *m == PresentMode::AutoVsync)
            .next()
            .unwrap_or(surface_capabilities.present_modes[0]);

        SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            format: surface_format,
            width,
            height,
            present_mode,
            desired_maximum_frame_latency: 0,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        }
    }

    fn clear_surface(&self, context: &RenderingContext) {
        context
            .encoder()
            .borrow_mut()
            .as_mut()
            .unwrap()
            .begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &context.surface_view(),
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(self.settings.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
    }
}
