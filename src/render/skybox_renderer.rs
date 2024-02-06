use bytemuck::{bytes_of, cast_slice, Pod, Zeroable};
use glam::{Mat3, Mat4, Vec3};
use once_cell::sync::Lazy;
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferBinding,
    BufferBindingType, BufferUsages, ColorTargetState, ColorWrites, CommandEncoder, Device, Face,
    FragmentState, FrontFace, IndexFormat, LoadOp, MultisampleState, Operations, PipelineLayout,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderModule, ShaderStages, StoreOp, TextureView, VertexState,
};

use super::{render_manager::RenderManager, renderer::Renderer, scene::Camera, vertex::Vertex};

#[derive(Clone, Copy)]
pub struct SkyboxRendererSettings {
    pub sky_color: Vec3,
    pub horizon_color: Vec3,
    pub bottom_color: Vec3,
    pub scattering: f32,
}

impl Default for SkyboxRendererSettings {
    fn default() -> Self {
        Self {
            sky_color: Vec3::new(0.17, 0.49, 0.988),
            horizon_color: Vec3::new(0.72, 0.9, 0.96),
            bottom_color: Vec3::new(0.15, 0.47, 0.76),
            scattering: 0.45,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct SkyboxUniform {
    pub transform_matrix: Mat4,
    pub sky_color: Vec3,
    _padding1: f32,
    pub horizon_color: Vec3,
    _padding2: f32,
    pub bottom_color: Vec3,
    pub scattering: f32,
}

impl SkyboxUniform {
    pub fn new(
        sky_color: Vec3,
        horizon_color: Vec3,
        bottom_color: Vec3,
        scattering: f32,
    ) -> SkyboxUniform {
        SkyboxUniform {
            sky_color,
            horizon_color,
            bottom_color,
            scattering,
            ..Default::default()
        }
    }
}

static SKYBOX_VERTICES: Lazy<[Vertex; 24]> = Lazy::new(|| {
    [
        // Front face
        Vertex::new(Vec3::new(-1.0, -1.0, 1.0), Vec3::NEG_Z, Vec3::ONE),
        Vertex::new(Vec3::new(1.0, -1.0, 1.0), Vec3::NEG_Z, Vec3::ONE),
        Vertex::new(Vec3::new(1.0, 1.0, 1.0), Vec3::NEG_Z, Vec3::ONE),
        Vertex::new(Vec3::new(-1.0, 1.0, 1.0), Vec3::NEG_Z, Vec3::ONE),
        // Left face
        Vertex::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::X, Vec3::ONE),
        Vertex::new(Vec3::new(-1.0, -1.0, 1.0), Vec3::X, Vec3::ONE),
        Vertex::new(Vec3::new(-1.0, 1.0, 1.0), Vec3::X, Vec3::ONE),
        Vertex::new(Vec3::new(-1.0, 1.0, -1.0), Vec3::X, Vec3::ONE),
        // Back face
        Vertex::new(Vec3::new(1.0, -1.0, -1.0), Vec3::Z, Vec3::ONE),
        Vertex::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::Z, Vec3::ONE),
        Vertex::new(Vec3::new(-1.0, 1.0, -1.0), Vec3::Z, Vec3::ONE),
        Vertex::new(Vec3::new(1.0, 1.0, -1.0), Vec3::Z, Vec3::ONE),
        // Right face
        Vertex::new(Vec3::new(1.0, -1.0, 1.0), Vec3::NEG_X, Vec3::ONE),
        Vertex::new(Vec3::new(1.0, -1.0, -1.0), Vec3::NEG_X, Vec3::ONE),
        Vertex::new(Vec3::new(1.0, 1.0, -1.0), Vec3::NEG_X, Vec3::ONE),
        Vertex::new(Vec3::new(1.0, 1.0, 1.0), Vec3::NEG_X, Vec3::ONE),
        // Top face
        Vertex::new(Vec3::new(-1.0, 1.0, 1.0), Vec3::NEG_Y, Vec3::ONE),
        Vertex::new(Vec3::new(1.0, 1.0, 1.0), Vec3::NEG_Y, Vec3::ONE),
        Vertex::new(Vec3::new(1.0, 1.0, -1.0), Vec3::NEG_Y, Vec3::ONE),
        Vertex::new(Vec3::new(-1.0, 1.0, -1.0), Vec3::NEG_Y, Vec3::ONE),
        // Bottom face
        Vertex::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::NEG_Y, Vec3::ONE),
        Vertex::new(Vec3::new(1.0, -1.0, -1.0), Vec3::NEG_Y, Vec3::ONE),
        Vertex::new(Vec3::new(1.0, -1.0, 1.0), Vec3::NEG_Y, Vec3::ONE),
        Vertex::new(Vec3::new(-1.0, -1.0, 1.0), Vec3::NEG_Y, Vec3::ONE),
    ]
});

static SKYBOX_INDICES: [u16; 36] = [
    0, 1, 2, 2, 3, 0, // Front face
    4, 5, 6, 6, 7, 4, // Left face
    8, 9, 10, 10, 11, 8, // Back face
    12, 13, 14, 14, 15, 12, // Right face
    16, 17, 18, 18, 19, 16, // Top face
    20, 21, 22, 22, 23, 20, // Bottom face
];

pub struct SkyboxRenderer {
    shader: ShaderModule,
    pipeline_layout: PipelineLayout,
    pipeline: RenderPipeline,

    vertex_buffer: Buffer,
    index_buffer: Buffer,

    uniform: SkyboxUniform,
    uniform_buffer: Buffer,
    uniform_bind_group_layout: BindGroupLayout,
    uniform_bind_group: BindGroup,
}

impl SkyboxRenderer {
    pub fn new(
        settings: &SkyboxRendererSettings,
        render_manager: &RenderManager,
    ) -> SkyboxRenderer {
        let device = render_manager.device();

        let uniform = SkyboxUniform::new(
            settings.sky_color,
            settings.horizon_color,
            settings.bottom_color,
            settings.scattering,
        );

        let (uniform_buffer, uniform_bind_group_layout, uniform_bind_group) =
            Self::create_uniform(&uniform, device);

        let shader = device.create_shader_module(include_wgsl!("../shaders/skybox.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                render_manager.scene_bind_group_layout(),
                &uniform_bind_group_layout,
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
            depth_stencil: None,
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

        SkyboxRenderer {
            shader,
            pipeline_layout,
            pipeline,

            vertex_buffer: Self::create_vertex_buffer(device),
            index_buffer: Self::create_index_buffer(device),

            uniform,
            uniform_buffer,
            uniform_bind_group_layout,
            uniform_bind_group,
        }
    }

    fn create_uniform(
        uniform: &SkyboxUniform,
        device: &Device,
    ) -> (Buffer, BindGroupLayout, BindGroup) {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytes_of(uniform),
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

        (buffer, bind_group_layout, bind_group)
    }

    fn create_vertex_buffer(device: &Device) -> Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: cast_slice(SKYBOX_VERTICES.as_ref()),
            usage: BufferUsages::VERTEX,
        })
    }

    fn create_index_buffer(device: &Device) -> Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: cast_slice(&SKYBOX_INDICES),
            usage: BufferUsages::INDEX,
        })
    }
}

impl Renderer for SkyboxRenderer {
    fn render(
        &mut self,
        camera: &mut Camera,
        surface_view: &TextureView,
        depth_view: &TextureView,
        scene_bind_group: &BindGroup,
        queue: &mut Queue,
        encoder: &mut CommandEncoder,
    ) {
        self.uniform.transform_matrix =
            camera.proj_matrix() * Mat4::from_mat3(Mat3::from_mat4(camera.view_matrix()));

        queue.write_buffer(&self.uniform_buffer, 0, bytes_of(&self.uniform));

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: surface_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
        pass.set_bind_group(0, scene_bind_group, &[]);
        pass.set_bind_group(1, &self.uniform_bind_group, &[]);

        pass.draw_indexed(0..(SKYBOX_INDICES.len() as u32), 0, 0..1);
    }
}
