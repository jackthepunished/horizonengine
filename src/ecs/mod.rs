//! Entity Component System module
//!
//! Built on top of the hecs ECS library

mod components;
mod world;

pub use components::{Name, Transform, Velocity};
pub use world::World;
