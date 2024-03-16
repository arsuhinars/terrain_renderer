use std::rc::Rc;

use bytemuck::{bytes_of, Pod, Zeroable};
use glam::{Mat4, Quat, Vec2, Vec3};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
    BufferBinding, BufferBindingType, BufferUsages, Device, FilterMode, Queue, Sampler,
    SamplerBindingType, SamplerDescriptor, ShaderStages, Texture, TextureSampleType, TextureView,
    TextureViewDimension,
};

use super::bind_group::BindGroupHelper;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GlobalLight {
    pub light_direction: Vec3,
    _padding1: f32,
    pub light_color: Vec3,
    _padding2: f32,
}

impl GlobalLight {
    pub fn new(light_direction: Vec3, light_color: Vec3) -> GlobalLight {
        GlobalLight {
            light_direction,
            light_color,
            ..Default::default()
        }
    }
}

impl Default for GlobalLight {
    fn default() -> Self {
        Self {
            light_direction: Vec3::new(-1.0, -1.0, -1.0),
            _padding1: Default::default(),
            light_color: Vec3::new(0.8, 0.48, 0.74),
            _padding2: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct SceneUniform {
    pub view_proj_matrix: Mat4,
    pub camera_dir: Vec3,
    _padding1: f32,
    pub camera_pos: Vec3,
    _padding2: f32,
    pub surface_size: Vec2,
    pub camera_near: f32,
    pub camera_far: f32,
    pub global_light: GlobalLight,
    pub ambient_light: Vec3,
    pub time: f32,
}

impl SceneUniform {
    pub fn new(
        view_proj_matrix: Mat4,
        global_light: GlobalLight,
        ambient_light: Vec3,
    ) -> SceneUniform {
        SceneUniform {
            view_proj_matrix,
            global_light,
            ambient_light,
            ..Default::default()
        }
    }
}

impl Default for SceneUniform {
    fn default() -> Self {
        Self {
            view_proj_matrix: Default::default(),
            camera_dir: Default::default(),
            _padding1: Default::default(),
            camera_pos: Default::default(),
            _padding2: Default::default(),
            surface_size: Default::default(),
            camera_near: Default::default(),
            camera_far: Default::default(),
            global_light: Default::default(),
            ambient_light: Vec3::new(0.085, 0.245, 0.494),
            time: 0.0,
        }
    }
}

pub struct SceneBindGroup {
    uniform: Box<SceneUniform>,
    opaque_sampler: Sampler,
    opaque_texture: Texture,
    opaque_view: TextureView,
    opaque_depth_texture: Texture,
    opaque_depth_view: TextureView,

    buffer: Buffer,
    layout: BindGroupLayout,
    bind_group: Option<Rc<BindGroup>>,
}

impl SceneBindGroup {
    pub fn new(
        device: &Device,
        opaque_texture: Texture,
        opaque_depth_texture: Texture,
    ) -> SceneBindGroup {
        let uniform = Box::new(SceneUniform::default());

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytes_of(uniform.as_ref()),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let opaque_sampler = device.create_sampler(&SamplerDescriptor {
            label: None,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 1.0,
            lod_max_clamp: 1.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        Self {
            uniform,
            opaque_sampler,
            opaque_view: opaque_texture.create_view(&Default::default()),
            opaque_texture,
            opaque_depth_view: opaque_depth_texture.create_view(&Default::default()),
            opaque_depth_texture,

            buffer,
            layout: Self::create_bind_group_layout(device),
            bind_group: None,
        }
    }

    pub fn uniform(&self) -> &SceneUniform {
        &self.uniform
    }

    pub fn update_uniform(&mut self, queue: &Queue, uniform: &SceneUniform) {
        *self.uniform.as_mut() = *uniform;
        queue.write_buffer(&self.buffer, 0, bytes_of(uniform));
    }

    pub fn opaque_texture(&self) -> &Texture {
        &self.opaque_texture
    }

    pub fn opaque_texture_view(&self) -> &TextureView {
        &self.opaque_view
    }

    pub fn opaque_depth_texture(&self) -> &Texture {
        &self.opaque_depth_texture
    }

    pub fn opaque_depth_view(&self) -> &TextureView {
        &self.opaque_depth_view
    }

    pub fn update_textures(&mut self, opaque_texture: Texture, opaque_depth_texture: Texture) {
        self.opaque_texture = opaque_texture;
        self.opaque_view = self.opaque_texture.create_view(&Default::default());
        self.opaque_depth_texture = opaque_depth_texture;
        self.opaque_depth_view = self.opaque_depth_texture.create_view(&Default::default());

        self.bind_group = None;
    }

    fn create_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Depth,
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        })
    }

    fn create_bind_group(&self, device: &Device) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &self.buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.opaque_sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&self.opaque_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&self.opaque_depth_view),
                },
            ],
        })
    }
}

impl BindGroupHelper for SceneBindGroup {
    fn layout(&self) -> &BindGroupLayout {
        &self.layout
    }

    fn bind_group(&mut self, device: &Device) -> Rc<BindGroup> {
        if self.bind_group.is_none() {
            self.bind_group
                .replace(Rc::new(self.create_bind_group(device)));
        }

        self.bind_group.as_ref().unwrap().clone()
    }
}

pub struct Camera {
    position: Vec3,
    rotation: Quat,
    fov: f32,
    aspect_ratio: f32,
    near_plane: f32,
    far_plane: f32,
    is_dirty: bool,
    look_dir: Vec3,
    view_matrix: Mat4,
    proj_matrix: Mat4,
    view_proj_matrix: Mat4,
}

impl Camera {
    pub fn new(
        position: Vec3,
        rotation: Quat,
        fov: f32,
        aspect_ratio: f32,
        near_plane: f32,
        far_plane: f32,
    ) -> Camera {
        Camera {
            position,
            rotation,
            fov,
            aspect_ratio,
            near_plane,
            far_plane,
            is_dirty: true,
            look_dir: Default::default(),
            view_matrix: Default::default(),
            proj_matrix: Default::default(),
            view_proj_matrix: Default::default(),
        }
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.is_dirty = true;
    }

    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        self.is_dirty = true;
    }

    pub fn fov(&self) -> f32 {
        self.fov
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
        self.is_dirty = true;
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
        self.is_dirty = true;
    }

    pub fn near_plane(&self) -> f32 {
        self.near_plane
    }

    pub fn set_near_plane(&mut self, near_plane: f32) {
        self.near_plane = near_plane;
        self.is_dirty = true;
    }

    pub fn far_plane(&self) -> f32 {
        self.far_plane
    }

    pub fn set_far_plane(&mut self, far_plane: f32) {
        self.far_plane = far_plane;
        self.is_dirty = true;
    }

    pub fn look_dir(&mut self) -> Vec3 {
        if self.is_dirty {
            self.update_values();
            self.is_dirty = false;
        }

        self.look_dir
    }

    pub fn view_matrix(&mut self) -> Mat4 {
        if self.is_dirty {
            self.update_values();
            self.is_dirty = false;
        }

        self.view_matrix
    }

    pub fn proj_matrix(&mut self) -> Mat4 {
        if self.is_dirty {
            self.update_values();
            self.is_dirty = false;
        }

        self.proj_matrix
    }

    pub fn view_proj_matrix(&mut self) -> Mat4 {
        if self.is_dirty {
            self.update_values();
            self.is_dirty = false;
        }

        self.view_proj_matrix
    }

    fn update_values(&mut self) {
        self.look_dir = self.rotation.mul_vec3(Vec3::Z);
        self.view_matrix = Mat4::from_rotation_translation(self.rotation, self.position).inverse();
        self.proj_matrix = Mat4::perspective_lh(
            self.fov.to_radians(),
            self.aspect_ratio,
            self.near_plane,
            self.far_plane,
        );
        self.view_proj_matrix = self.proj_matrix * self.view_matrix;
    }
}
