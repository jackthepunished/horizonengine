//! Common ECS components

use glam::{Mat4, Quat, Vec3};

/// Transform component for position, rotation, and scale
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    /// Position in world space
    pub position: Vec3,
    /// Rotation as a quaternion
    pub rotation: Quat,
    /// Scale factor
    pub scale: Vec3,
}

impl Transform {
    /// Create a new transform at the origin
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a transform with just a position
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Create a transform with position and rotation
    pub fn from_position_rotation(position: Vec3, rotation: Quat) -> Self {
        Self {
            position,
            rotation,
            ..Default::default()
        }
    }

    /// Get the transformation matrix
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Get the forward direction (negative Z in local space)
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    /// Get the right direction (positive X in local space)
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// Get the up direction (positive Y in local space)
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// Translate by a delta
    pub fn translate(&mut self, delta: Vec3) {
        self.position += delta;
    }

    /// Rotate by euler angles (in radians)
    pub fn rotate_euler(&mut self, euler: Vec3) {
        self.rotation =
            Quat::from_euler(glam::EulerRot::XYZ, euler.x, euler.y, euler.z) * self.rotation;
    }

    /// Look at a target position
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let forward = (target - self.position).normalize();
        self.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, forward);
        // Adjust for up vector
        let right = forward.cross(up).normalize();
        let adjusted_up = right.cross(forward);
        self.rotation =
            Quat::from_mat4(&Mat4::look_at_rh(self.position, target, adjusted_up)).inverse();
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

/// Velocity component for physics
#[derive(Debug, Clone, Copy, Default)]
pub struct Velocity {
    pub linear: Vec3,
    pub angular: Vec3,
}

/// Name component for debugging
#[derive(Debug, Clone)]
pub struct Name(pub String);

impl Name {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}
