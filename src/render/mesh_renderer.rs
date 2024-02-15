use wgpu::{
    include_wgsl, BlendState, Buffer, ColorTargetState, ColorWrites, CompareFunction,
    DepthBiasState, DepthStencilState, Face, FragmentState, FrontFace, IndexFormat, LoadOp,
    MultisampleState, Operations, PipelineLayout, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderModule, StencilFaceState,
    StencilState, StoreOp, VertexState,
};

use super::{
    mesh::Mesh,
    render_manager::RenderManager,
    renderer::{Renderer, RenderingContext},
    vertex::Vertex,
};

pub struct MeshRenderer {
    shader: ShaderModule,
    pipeline_layout: PipelineLayout,
    pipeline: RenderPipeline,
    mesh: Mesh,
}

impl MeshRenderer {
    pub fn new(mesh: Mesh, render_manager: &RenderManager) -> MeshRenderer {
        let device = render_manager.device();

        let shader = device.create_shader_module(include_wgsl!("../shaders/mesh.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[render_manager.scene_bind_group_layout()],
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
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::COLOR,
                })],
            }),
            multiview: None,
        });

        MeshRenderer {
            shader,
            pipeline_layout,
            pipeline,
            mesh,
        }
    }
}

impl Renderer for MeshRenderer {
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

        pass.draw_indexed(0..(self.mesh.indices().len() as u32), 0, 0..1);
    }
}
