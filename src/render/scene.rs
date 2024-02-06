use bytemuck::{Pod, Zeroable};
use glam::{Mat3, Mat4, Quat, Vec3};

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct SceneUniform {
    pub view_proj_matrix: Mat4,
    pub global_light: GlobalLight,
}

impl SceneUniform {
    pub fn new(view_proj_matrix: Mat4, global_light: GlobalLight) -> SceneUniform {
        SceneUniform {
            view_proj_matrix,
            global_light,
            ..Default::default()
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
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

pub struct Camera {
    position: Vec3,
    rotation: Quat,
    fov: f32,
    aspect_ratio: f32,
    near_plane: f32,
    far_plane: f32,
    is_dirty: bool,
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

    pub fn view_matrix(&mut self) -> Mat4 {
        if self.is_dirty {
            self.update_matrices();
            self.is_dirty = false;
        }

        self.view_matrix
    }

    pub fn proj_matrix(&mut self) -> Mat4 {
        if self.is_dirty {
            self.update_matrices();
            self.is_dirty = false;
        }

        self.proj_matrix
    }

    pub fn view_proj_matrix(&mut self) -> Mat4 {
        if self.is_dirty {
            self.update_matrices();
            self.is_dirty = false;
        }

        self.view_proj_matrix
    }

    fn update_matrices(&mut self) {
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
