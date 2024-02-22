use bytemuck::{bytes_of, cast_slice, Pod, Zeroable};
use glam::{Vec2, Vec3};
use half::f16;
use noise::{Constant, NoiseFn, Perlin};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt, TextureDataOrder},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferBinding,
    BufferBindingType, BufferUsages, ColorTargetState, ColorWrites, CompareFunction,
    DepthBiasState, DepthStencilState, Device, Extent3d, Face, FragmentState, FrontFace,
    IndexFormat, LoadOp, MultisampleState, Operations, PipelineLayout, PipelineLayoutDescriptor,
    PolygonMode, PrimitiveState, PrimitiveTopology, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, ShaderStages, StencilFaceState, StencilState, StoreOp,
    Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureView, TextureViewDimension, VertexState,
};

use crate::utils::terrain_generator::{generate_terrain_mesh, TerrainSettings};

use super::{
    mesh::Mesh,
    render_manager::RenderManager,
    renderer::{Renderer, RenderingContext},
    vertex::Vertex,
};

pub struct WaterRendererSettings {
    pub tile_size: f32,
    pub tiles_count: u32,
    pub color: Vec3,
    pub specular: f32,
    pub alpha: f32,
    pub level: f32,
    pub wave_speed: Vec2,
    pub wave_scale: Vec2,
    pub wave_height: f32,
    pub wave_texture_size: u32,
    pub wave_texture_scale: f32,
}

pub struct WaterRenderer {
    _shader: ShaderModule,
    _pipeline_layout: PipelineLayout,
    pipeline: RenderPipeline,

    mesh: Mesh,

    uniform_buffer: Buffer,
    _wave_texture: Texture,
    _wave_texture_view: TextureView,
    _bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
struct WaterUniform {
    pub specular: f32,
    pub alpha: f32,
    pub wave_speed: Vec2,
    pub wave_scale: Vec2,
    pub wave_height: f32,
}

impl WaterRenderer {
    pub fn new(settings: &WaterRendererSettings, render_manager: &RenderManager) -> WaterRenderer {
        let device = render_manager.device();

        let shader = device.create_shader_module(include_wgsl!("../shaders/water.glsl"));

        let uniform = WaterUniform {
            specular: settings.specular,
            alpha: settings.alpha,
            wave_speed: settings.wave_speed,
            wave_scale: settings.wave_scale,
            wave_height: settings.wave_height,
        };
        let wave_texture = device.create_texture_with_data(
            &render_manager.queue().borrow(),
            &TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: settings.wave_texture_size,
                    height: settings.wave_texture_size,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R16Float,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[TextureFormat::R16Float],
            },
            TextureDataOrder::LayerMajor,
            cast_slice(
                Self::create_noise_texture(settings.wave_texture_size, settings.wave_texture_scale)
                    .as_ref(),
            ),
        );
        let wave_texture_view = wave_texture.create_view(&Default::default());
        let (uniform_buffer, bind_group_layout, bind_group) =
            Self::create_uniform(&uniform, &wave_texture_view, device);

        let mesh = generate_terrain_mesh(
            device,
            &TerrainSettings {
                tile_size: settings.tile_size,
                tiles_count: settings.tiles_count,
                colors: vec![settings.color].into_boxed_slice(),
                colors_thresholds: vec![].into_boxed_slice(),
                noise: Constant::new(settings.level.into()),
                scale: 1.0,
                max_height: 1.0,
            },
        );

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[render_manager.scene_bind_group_layout(), &bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::buffer_layout()],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: render_manager.depth_texture().format(),
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: render_manager.surface_format(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::COLOR,
                })],
            }),
            multiview: None,
        });

        WaterRenderer {
            _shader: shader,
            _pipeline_layout: pipeline_layout,
            pipeline,

            mesh,

            uniform_buffer,
            _wave_texture: wave_texture,
            _wave_texture_view: wave_texture_view,
            _bind_group_layout: bind_group_layout,
            bind_group,
        }
    }

    fn create_uniform(
        uniform: &WaterUniform,
        wave_texture: &TextureView,
        device: &Device,
    ) -> (Buffer, BindGroupLayout, BindGroup) {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytes_of(uniform),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::all(),
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::all(),
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(wave_texture),
                },
            ],
        });

        (buffer, bind_group_layout, bind_group)
    }

    fn create_noise_texture(size: u32, scale: f32) -> Box<[f16]> {
        let noise = Perlin::new(Perlin::DEFAULT_SEED);

        let mut v = Vec::<f16>::new();
        v.resize((size * size) as usize, f16::ZERO);

        for x in 0..size {
            for y in 0..size {
                let p = Vec2::new(x as f32, y as f32) * scale;
                v[(y * size + x) as usize] = f16::from_f64(noise.get(p.as_dvec2().into()));
            }
        }

        v.into_boxed_slice()
    }
}

impl Renderer for WaterRenderer {
    fn render(&mut self, context: &RenderingContext) {
        let mut encoder_ref = context.encoder().borrow_mut();
        let encoder = encoder_ref.as_mut().unwrap();

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: context.surface_view(),
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: context.depth_view(),
                depth_ops: Some(Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.mesh.vertex_buffer().slice(..));
        pass.set_index_buffer(self.mesh.index_buffer().slice(..), IndexFormat::Uint16);
        pass.set_bind_group(0, context.scene_bind_group(), &[]);
        pass.set_bind_group(1, &self.bind_group, &[]);

        pass.draw_indexed(0..(self.mesh.indices().len() as u32), 0, 0..1);
    }
}
