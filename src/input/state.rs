//! Input handling

use glam::Vec2;
use std::collections::HashSet;
use winit::event::{ElementState, MouseButton};
use winit::keyboard::KeyCode;

/// Input state manager
#[derive(Debug)]
pub struct Input {
    /// Currently pressed keys
    pressed_keys: HashSet<KeyCode>,
    /// Keys that were just pressed this frame
    just_pressed_keys: HashSet<KeyCode>,
    /// Keys that were just released this frame
    just_released_keys: HashSet<KeyCode>,
    /// Currently pressed mouse buttons
    pressed_mouse_buttons: HashSet<MouseButton>,
    /// Mouse buttons just pressed this frame
    just_pressed_mouse_buttons: HashSet<MouseButton>,
    /// Mouse buttons just released this frame
    just_released_mouse_buttons: HashSet<MouseButton>,
    /// Current mouse position
    mouse_position: Vec2,
    /// Mouse movement delta this frame
    mouse_delta: Vec2,
    /// Scroll wheel delta this frame
    scroll_delta: Vec2,
}

impl Input {
    /// Create a new input manager
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            just_pressed_keys: HashSet::new(),
            just_released_keys: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
            just_pressed_mouse_buttons: HashSet::new(),
            just_released_mouse_buttons: HashSet::new(),
            mouse_position: Vec2::ZERO,
            mouse_delta: Vec2::ZERO,
            scroll_delta: Vec2::ZERO,
        }
    }

    /// Call at the start of each frame to clear per-frame state
    pub fn update(&mut self) {
        self.just_pressed_keys.clear();
        self.just_released_keys.clear();
        self.just_pressed_mouse_buttons.clear();
        self.just_released_mouse_buttons.clear();
        self.mouse_delta = Vec2::ZERO;
        self.scroll_delta = Vec2::ZERO;
    }

    /// Process a keyboard event
    pub fn process_keyboard(&mut self, key_code: KeyCode, state: ElementState) {
        match state {
            ElementState::Pressed => {
                if !self.pressed_keys.contains(&key_code) {
                    self.just_pressed_keys.insert(key_code);
                }
                self.pressed_keys.insert(key_code);
            }
            ElementState::Released => {
                self.pressed_keys.remove(&key_code);
                self.just_released_keys.insert(key_code);
            }
        }
    }

    /// Process a mouse button event
    pub fn process_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => {
                if !self.pressed_mouse_buttons.contains(&button) {
                    self.just_pressed_mouse_buttons.insert(button);
                }
                self.pressed_mouse_buttons.insert(button);
            }
            ElementState::Released => {
                self.pressed_mouse_buttons.remove(&button);
                self.just_released_mouse_buttons.insert(button);
            }
        }
    }

    /// Process mouse movement
    pub fn process_mouse_motion(&mut self, position: Vec2) {
        self.mouse_delta = position - self.mouse_position;
        self.mouse_position = position;
    }

    /// Process raw mouse delta (for first-person camera)
    pub fn process_mouse_delta(&mut self, delta: Vec2) {
        self.mouse_delta += delta;
    }

    /// Process scroll wheel
    pub fn process_scroll(&mut self, delta: Vec2) {
        self.scroll_delta += delta;
    }

    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    /// Check if a key was just pressed this frame
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed_keys.contains(&key)
    }

    /// Check if a key was just released this frame
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.just_released_keys.contains(&key)
    }

    /// Check if a mouse button is currently pressed
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_mouse_buttons.contains(&button)
    }

    /// Check if a mouse button was just pressed this frame
    pub fn is_mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        self.just_pressed_mouse_buttons.contains(&button)
    }

    /// Check if a mouse button was just released this frame
    pub fn is_mouse_button_just_released(&self, button: MouseButton) -> bool {
        self.just_released_mouse_buttons.contains(&button)
    }

    /// Get current mouse position
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    /// Get mouse movement delta this frame
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    /// Get scroll wheel delta this frame
    pub fn scroll_delta(&self) -> Vec2 {
        self.scroll_delta
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}
