//! Physics simulation module
//!
//! Built on top of rapier3d

mod physics;

pub use physics::{ColliderHandle, Physics, RaycastHit, RigidBodyHandle};
