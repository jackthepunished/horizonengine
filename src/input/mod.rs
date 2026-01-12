//! Input handling module
//!
//! Provides raw input state tracking and command pattern for input abstraction.

mod command;
mod state;

pub use command::{Command, CommandHistory, InputAction, InputMapper};
pub use state::Input;
