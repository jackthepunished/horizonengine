//! A 3D Game Engine built in Rust
//!
//! This engine provides:
//! - 3D rendering with wgpu
//! - Entity Component System (ECS) architecture
//! - Physics simulation with rapier3d
//! - Input handling with winit

pub mod core;
pub mod ecs;
pub mod input;
pub mod physics;
pub mod renderer;

// Re-exports for convenience
pub use glam;
pub use hecs;
pub use rapier3d;
pub use wgpu;
pub use winit;

/// Prelude module for common imports
pub mod prelude {
    pub use crate::core::{DebugInfo, Engine, EngineConfig, EngineContext, FrameStats, Game};
    pub use crate::ecs::{Name, Transform, Velocity, World};
    pub use crate::input::Input;
    pub use crate::physics::{ColliderHandle, Physics, RigidBodyHandle};
    pub use crate::renderer::{Camera, Light, Material, Mesh, RenderFrame, Renderer, Vertex};
    pub use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
    pub use winit::keyboard::KeyCode;
}
