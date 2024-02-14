use glam::{Quat, Vec2, Vec3};

use crate::{
    core::{input_manager::InputManager, time_manager::TimeManager},
    render::render_manager::RenderManager,
};

#[derive(Clone, Copy)]
pub struct CameraSettings {
    initial_pos: Vec3,
    initial_rotation_angles: Vec2,
    speed: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            initial_pos: Vec3::ZERO,
            initial_rotation_angles: Vec2::ZERO,
            speed: 1.0,
        }
    }
}

pub struct CameraController {
    settings: CameraSettings,
    position: Vec3,
    rotation_angles: Vec2,
}

impl CameraController {
    pub fn new(settings: &CameraSettings) -> CameraController {
        Self {
            settings: *settings,
            position: settings.initial_pos,
            rotation_angles: settings.initial_rotation_angles,
        }
    }

    pub fn update(
        &mut self,
        time_manager: &TimeManager,
        input_manager: &InputManager,
        render_manager: &mut RenderManager,
    ) {
        self.rotation_angles += input_manager.look_delta();

        let rotation = Quat::from_rotation_y(self.rotation_angles.x.to_radians())
            * Quat::from_rotation_x(self.rotation_angles.y.to_radians());

        self.position += self.settings.speed
            * time_manager.delta()
            * rotation.mul_vec3(input_manager.move_vector());

        let mut camera = render_manager.camera().borrow_mut();

        camera.set_position(self.position);
        camera.set_rotation(rotation);
    }
}
