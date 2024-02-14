use glam::{Vec2, Vec3};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Clone, Copy)]
pub struct InputSettings<'a> {
    look_sensitivity: f32,
    right_keys: &'a [PhysicalKey],
    left_keys: &'a [PhysicalKey],
    up_keys: &'a [PhysicalKey],
    down_keys: &'a [PhysicalKey],
    forward_keys: &'a [PhysicalKey],
    backward_keys: &'a [PhysicalKey],
}

impl<'a> Default for InputSettings<'a> {
    fn default() -> Self {
        Self {
            look_sensitivity: 0.1,
            right_keys: &[
                PhysicalKey::Code(KeyCode::KeyD),
                PhysicalKey::Code(KeyCode::ArrowRight),
            ],
            left_keys: &[
                PhysicalKey::Code(KeyCode::KeyA),
                PhysicalKey::Code(KeyCode::ArrowLeft),
            ],
            up_keys: &[PhysicalKey::Code(KeyCode::KeyE)],
            down_keys: &[PhysicalKey::Code(KeyCode::KeyD)],
            forward_keys: &[
                PhysicalKey::Code(KeyCode::KeyW),
                PhysicalKey::Code(KeyCode::ArrowUp),
            ],
            backward_keys: &[
                PhysicalKey::Code(KeyCode::KeyS),
                PhysicalKey::Code(KeyCode::ArrowDown),
            ],
        }
    }
}

pub struct InputManager<'a> {
    settings: Box<InputSettings<'a>>,
    last_cursor_pos: Vec2,
    move_vector: Vec3,
    look_delta: Vec2,
}

impl<'a> InputManager<'a> {
    pub fn new(settings: &InputSettings<'a>) -> InputManager<'a> {
        InputManager {
            settings: Box::new(*settings),
            last_cursor_pos: Default::default(),
            move_vector: Vec3::ZERO,
            look_delta: Vec2::ZERO,
        }
    }

    pub fn handle_keyboard_input(&mut self, event: KeyEvent) {
        let key = event.physical_key;

        match event.state {
            ElementState::Pressed => {
                if self.settings.right_keys.contains(&key) {
                    self.move_vector.x = 1.0;
                } else if self.settings.left_keys.contains(&key) {
                    self.move_vector.x = -1.0;
                }

                if self.settings.up_keys.contains(&key) {
                    self.move_vector.y = 1.0;
                } else if self.settings.down_keys.contains(&key) {
                    self.move_vector.y = -1.0;
                }

                if self.settings.forward_keys.contains(&key) {
                    self.move_vector.z = 1.0;
                } else if self.settings.backward_keys.contains(&key) {
                    self.move_vector.z = -1.0;
                }
            }
            ElementState::Released => {
                if self.settings.right_keys.contains(&key) || self.settings.left_keys.contains(&key)
                {
                    self.move_vector.x = 0.0;
                } else if self.settings.up_keys.contains(&key)
                    || self.settings.down_keys.contains(&key)
                {
                    self.move_vector.z = 0.0;
                } else if self.settings.forward_keys.contains(&key)
                    || self.settings.backward_keys.contains(&key)
                {
                    self.move_vector.y = 0.0;
                }
            }
        }
    }

    pub fn handle_cursor_movement(&mut self, cursor_position: PhysicalPosition<f64>) {
        let cursor_pos: Vec2 = mint::Point2::from(cursor_position.cast::<f32>()).into();

        self.look_delta = (cursor_pos - self.last_cursor_pos) * self.settings.look_sensitivity;
        self.last_cursor_pos = cursor_pos;
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
