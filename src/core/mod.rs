//! Core engine module
//!
//! Contains the main Engine struct, configuration, and event system.

mod debug;
mod engine;
mod events;
mod scene;
mod time;

pub use debug::{DebugInfo, FrameStats};
pub use engine::{Engine, EngineConfig, EngineContext, Game};
pub use events::{EventQueue, GameEvent};
pub use scene::{Scene, SceneError, SerializedEntity};
pub use time::Time;
