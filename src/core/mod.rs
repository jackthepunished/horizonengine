//! Core engine module
//!
//! Contains the main Engine struct and configuration

mod debug;
mod engine;
mod scene;
mod time;

pub use debug::{DebugInfo, FrameStats};
pub use engine::{Engine, EngineConfig, EngineContext, Game};
pub use scene::{Scene, SceneError, SerializedEntity};
pub use time::Time;
