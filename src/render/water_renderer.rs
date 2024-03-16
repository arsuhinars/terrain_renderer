use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};
use noise::Constant;
use wgpu::{
    include_wgsl, BindGroup, BindGroupLayout, BlendState, Buffer, ColorTargetState, ColorWrites,
    CompareFunction, DepthBiasState, DepthStencilState, Face, FragmentState, FrontFace,
    IndexFormat, LoadOp, MultisampleState, Operations, PipelineLayout, PipelineLayoutDescriptor,
    PolygonMode, PrimitiveState, PrimitiveTopology, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, StencilFaceState, StencilState, StoreOp, VertexState,
};

use crate::utils::{
    create_uniform_init,
    terrain_generator::{generate_terrain_mesh, TerrainSettings},
};

use super::{
    bind_group::BindGroupHelper,
    mesh::Mesh,
    render_manager::RenderManager,
    renderer::{RenderStage, Renderer, RenderingContext},
    vertex::Vertex,
};

#[derive(Clone, Copy)]
pub struct WaterRendererSettings {
    pub tile_size: f32,
    pub tiles_count: u32,
    pub color: Vec3,
    pub specular: f32,
    pub specular_color: Vec3,
    pub density: f32,
    pub level: f32,
    pub wave_speed: Vec2,
    pub wave_scale: Vec2,
    pub wave_height: f32,
}

impl Default for WaterRendererSettings {
    fn default() -> Self {
        Self {
            tile_size: 0.75,
            tiles_count: 15,
            color: Vec3::new(0.2, 0.5, 0.96),
            specular: 64.0,
            specular_color: Vec3::new(0.75, 0.84, 0.97),
            density: 150.0,
            level: -0.25,
            wave_speed: Vec2::new(0.8, 0.4),
            wave_scale: Vec2::new(0.4, 0.4),
            wave_height: 0.2,
        }
    }
}

pub struct WaterRenderer {
    _shader: ShaderModule,
    _pipeline_layout: PipelineLayout,
    pipeline: RenderPipeline,

    mesh: Mesh,

    _uniform_buffer: Buffer,
    _bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
struct WaterUniform {
    pub specular: f32,
    pub density: f32,
    _padding1: [f32; 2],
    pub specular_color: Vec3,
    _padding2: f32,
    pub wave_speed: Vec2,
    pub wave_scale: Vec2,
    pub wave_height: f32,
    _padding3: [f32; 3],
}

impl WaterRenderer {
    pub fn new(settings: &WaterRendererSettings, render_manager: &RenderManager) -> WaterRenderer {
        let device = render_manager.device();

        let shader = device.create_shader_module(include_wgsl!("../shaders/water.glsl"));

        let uniform = WaterUniform {
            specular: settings.specular,
            density: settings.density,
            specular_color: settings.specular_color,
            wave_speed: settings.wave_speed,
            wave_scale: settings.wave_scale,
            wave_height: settings.wave_height,
            ..Default::default()
        };
        let (uniform_buffer, bind_group_layout, bind_group) = create_uniform_init(&uniform, device);

        let mesh: Mesh = generate_terrain_mesh(
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
            bind_group_layouts: &[
                render_manager.scene_bind_group().borrow().layout(),
                &bind_group_layout,
            ],
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

            _uniform_buffer: uniform_buffer,
            _bind_group_layout: bind_group_layout,
            bind_group,
        }
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

    fn stage(&self) -> RenderStage {
        RenderStage::TRANSPARENT
    }
}
