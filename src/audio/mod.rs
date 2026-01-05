//! Audio system for playing sounds and music
//!
//! Built on top of the rodio audio library.
//! Supports WAV, MP3, OGG, and FLAC formats.

mod manager;
mod source;

pub use manager::AudioManager;
pub use source::{AudioSource, PlaybackState};
