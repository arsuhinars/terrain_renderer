use std::{iter, sync::Arc};

use bytemuck::bytes_of;
use glam::{Quat, Vec3};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Adapter, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
    BufferBinding, BufferBindingType, BufferUsages, Color, Device, DeviceDescriptor, Extent3d,
    Instance, Operations, PresentMode, Queue, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RequestAdapterOptions, ShaderStages,
    Surface, SurfaceConfiguration, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureView,
};
use winit::{dpi::PhysicalSize, window::Window};

use super::{
    renderer::Renderer,
    scene::{Camera, GlobalLight, SceneUniform},
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
    settings: RenderSettings,
    surface_config: wgpu::SurfaceConfiguration,
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    depth_texture: Texture,
    depth_view: TextureView,

    camera: Camera,

    scene_uniform: SceneUniform,
    scene_buffer: Buffer,
    scene_bind_group_layout: BindGroupLayout,
    scene_bind_group: BindGroup,

    renderers: Vec<Box<dyn Renderer>>,
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

        let (depth_texture, depth_view) =
            Self::create_depth_texture(&device, surface_width, surface_height);

        let camera = Camera::new(
            Vec3::ZERO,
            Quat::IDENTITY,
            settings.camera_fov,
            (surface_width as f32) / (surface_height as f32),
            settings.camera_near_plane,
            settings.camera_far_plane,
        );

        let (scene_uniform, scene_buffer, scene_bind_group_layout, scene_bind_group) =
            Self::create_scene_uniform(&device);

        Ok(RenderManager {
            settings: *settings,
            surface_config,
            surface,
            device,
            queue,
            depth_texture,
            depth_view,

            camera,

            scene_uniform,
            scene_buffer,
            scene_bind_group_layout,
            scene_bind_group,

            renderers: Vec::new(),
        })
    }

    pub fn set_renderers(&mut self, renderers: Vec<Box<dyn Renderer>>) {
        self.renderers = renderers;
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.surface_config.format
    }

    pub fn depth_texture(&self) -> &Texture {
        &self.depth_texture
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn mut_camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn global_light(&self) -> &GlobalLight {
        &self.scene_uniform.global_light
    }

    pub fn mut_global_light(&mut self) -> &mut GlobalLight {
        &mut self.scene_uniform.global_light
    }

    pub fn scene_bind_group(&self) -> &BindGroup {
        &self.scene_bind_group
    }

    pub fn scene_bind_group_layout(&self) -> &BindGroupLayout {
        &self.scene_bind_group_layout
    }

    pub fn handle_resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);

        (self.depth_texture, self.depth_view) =
            Self::create_depth_texture(&self.device, size.width, size.height);

        self.camera
            .set_aspect_ratio((size.width as f32) / (size.height as f32));
    }

    pub fn render(&mut self) -> Result<(), String> {
        let surface_texture = self
            .surface
            .get_current_texture()
            .map_err(|err| err.to_string())?;
        let surface_view = surface_texture.texture.create_view(&Default::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());

        self.scene_uniform.view_proj_matrix = self.camera.view_proj_matrix();
        self.queue
            .write_buffer(&self.scene_buffer, 0, bytes_of(&self.scene_uniform));

        {
            encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(self.settings.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
        }

        for renderer in self.renderers.as_mut_slice() {
            renderer.render(
                &mut self.camera,
                &surface_view,
                &self.depth_view,
                &self.scene_bind_group,
                &mut self.queue,
                &mut encoder,
            );
        }

        self.queue.submit(iter::once(encoder.finish()));

        surface_texture.present();

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
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode,
            desired_maximum_frame_latency: 0,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        }
    }

    fn create_depth_texture(device: &Device, width: u32, height: u32) -> (Texture, TextureView) {
        let depth_texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&Default::default());

        (depth_texture, depth_view)
    }

    fn create_scene_uniform(device: &Device) -> (SceneUniform, Buffer, BindGroupLayout, BindGroup) {
        let scene_uniform = SceneUniform::default();

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytes_of(&scene_uniform),
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

        (scene_uniform, buffer, bind_group_layout, bind_group)
    }
}