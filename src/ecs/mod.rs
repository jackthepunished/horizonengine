//! Entity Component System module
//!
//! Built on top of the hecs ECS library.
//! Includes dirty flag pattern for cached transform computations.

mod cached;
mod components;
mod hierarchy;
mod world;

pub use cached::CachedTransform;
pub use components::{Name, Transform, Velocity};
pub use hierarchy::{Children, GlobalTransform, Parent};
pub use world::World;
