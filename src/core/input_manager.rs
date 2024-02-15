use glam::{Vec2, Vec3};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Clone, Copy)]
pub struct InputSettings {
    look_sensitivity: f32,
    right_key: PhysicalKey,
    left_key: PhysicalKey,
    up_key: PhysicalKey,
    down_key: PhysicalKey,
    forward_key: PhysicalKey,
    backward_key: PhysicalKey,
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            look_sensitivity: 0.1,
            right_key: PhysicalKey::Code(KeyCode::KeyD),
            left_key: PhysicalKey::Code(KeyCode::KeyA),
            up_key: PhysicalKey::Code(KeyCode::Space),
            down_key: PhysicalKey::Code(KeyCode::ControlLeft),
            forward_key: PhysicalKey::Code(KeyCode::KeyW),
            backward_key: PhysicalKey::Code(KeyCode::KeyS),
        }
    }
}

pub struct InputManager {
    settings: Box<InputSettings>,
    last_cursor_pos: Vec2,
    cursor_just_entered: bool,
    move_vector: Vec3,
    look_delta: Vec2,
}

impl InputManager {
    pub fn new(settings: &InputSettings) -> InputManager {
        InputManager {
            settings: Box::new(*settings),
            last_cursor_pos: Default::default(),
            cursor_just_entered: true,
            move_vector: Vec3::ZERO,
            look_delta: Vec2::ZERO,
        }
    }

    pub fn handle_keyboard_input(&mut self, event: KeyEvent) {
        let key = event.physical_key;

        match event.state {
            ElementState::Pressed => {
                if self.settings.right_key == key {
                    self.move_vector.x = 1.0;
                } else if self.settings.left_key == key {
                    self.move_vector.x = -1.0;
                } else if self.settings.up_key == key {
                    self.move_vector.y = 1.0;
                } else if self.settings.down_key == key {
                    self.move_vector.y = -1.0;
                } else if self.settings.forward_key == key {
                    self.move_vector.z = 1.0;
                } else if self.settings.backward_key == key {
                    self.move_vector.z = -1.0;
                }
            }
            ElementState::Released => {
                if self.settings.right_key == key || self.settings.left_key == key {
                    self.move_vector.x = 0.0;
                } else if self.settings.up_key == key || self.settings.down_key == key {
                    self.move_vector.y = 0.0;
                } else if self.settings.forward_key == key || self.settings.backward_key == key {
                    self.move_vector.z = 0.0;
                }
            }
        }
    }

    pub fn handle_cursor_movement(&mut self, cursor_position: PhysicalPosition<f64>) {
        let cursor_pos: Vec2 = mint::Point2::from(cursor_position.cast::<f32>()).into();

        self.look_delta = if self.cursor_just_entered {
            Vec2::ZERO
        } else {
            (cursor_pos - self.last_cursor_pos) * self.settings.look_sensitivity
        };
        self.last_cursor_pos = cursor_pos;
        self.cursor_just_entered = false;
    }

    pub fn handle_cursor_enter(&mut self) {
        self.cursor_just_entered = true;
    }

    pub fn late_update(&mut self) {
        self.look_delta = Vec2::ZERO;
    }

    pub fn move_vector(&self) -> Vec3 {
        self.move_vector
    }

    pub fn look_delta(&self) -> Vec2 {
        self.look_delta
    }
}
