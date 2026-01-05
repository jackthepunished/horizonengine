//! Physics simulation module
//!
//! Built on top of rapier3d

mod world;

pub use world::{ColliderHandle, Physics, RaycastHit, RigidBodyHandle};
