//! Camera system for 3D rendering

use glam::{Mat4, Vec3};

/// Perspective camera for 3D rendering
#[derive(Debug, Clone)]
pub struct Camera {
    /// Camera position in world space
    pub position: Vec3,
    /// Direction the camera is looking at
    pub direction: Vec3,
    /// Up vector
    pub up: Vec3,
    /// Field of view in radians
    pub fov: f32,
    /// Near clipping plane
    pub near: f32,
    /// Far clipping plane
    pub far: f32,
    /// Aspect ratio (width / height)
    pub aspect: f32,
    /// Yaw angle (rotation around Y axis)
    yaw: f32,
    /// Pitch angle (rotation around X axis)
    pitch: f32,
}

impl Camera {
    /// Create a new camera with default settings
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            direction: Vec3::NEG_Z,
            up: Vec3::Y,
            fov: std::f32::consts::FRAC_PI_4, // 45 degrees
            near: 0.1,
            far: 1000.0,
            aspect: 16.0 / 9.0,
            yaw: -90.0_f32.to_radians(),
            pitch: 0.0,
        }
    }

    /// Create a camera at a specific position looking at a target
    pub fn look_at(position: Vec3, target: Vec3, up: Vec3) -> Self {
        let direction = (target - position).normalize();
        let mut camera = Self::new();
        camera.position = position;
        camera.direction = direction;
        camera.up = up;

        // Calculate yaw and pitch from direction
        camera.yaw = direction.z.atan2(direction.x);
        camera.pitch = direction.y.asin();

        camera
    }

    /// Get the view matrix
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.direction, self.up)
    }

    /// Get the projection matrix
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }

    /// Get combined view-projection matrix
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Update aspect ratio
    pub fn set_aspect(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height.max(1) as f32;
    }

    /// Rotate camera using mouse delta
    pub fn rotate(&mut self, delta_x: f32, delta_y: f32, sensitivity: f32) {
        self.yaw += delta_x * sensitivity;
        self.pitch -= delta_y * sensitivity;

        // Clamp pitch to avoid gimbal lock
        let max_pitch = 89.0_f32.to_radians();
        self.pitch = self.pitch.clamp(-max_pitch, max_pitch);

        // Update direction from yaw and pitch
        self.direction = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();
    }

    /// Get the right vector
    pub fn right(&self) -> Vec3 {
        self.direction.cross(self.up).normalize()
    }

    /// Get the forward vector (same as direction)
    pub fn forward(&self) -> Vec3 {
        self.direction
    }

    /// Move camera forward/backward
    pub fn move_forward(&mut self, amount: f32) {
        self.position += self.direction * amount;
    }

    /// Move camera left/right
    pub fn move_right(&mut self, amount: f32) {
        self.position += self.right() * amount;
    }

    /// Move camera up/down (world space)
    pub fn move_up(&mut self, amount: f32) {
        self.position += Vec3::Y * amount;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
