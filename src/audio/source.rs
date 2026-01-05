//! Audio source for playing individual sounds

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

use rodio::{Decoder, Sink, mixer::Mixer};

/// Playback state of an audio source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackState {
    /// Audio is playing
    Playing,
    /// Audio is paused
    Paused,
    /// Audio has stopped
    #[default]
    Stopped,
}

/// An audio source that can play sounds
pub struct AudioSource {
    /// The audio sink for playback control
    sink: Sink,
    /// Current playback state
    state: PlaybackState,
    /// Whether this source loops
    looping: bool,
    /// Source name for debugging
    name: String,
}

impl AudioSource {
    /// Create a new audio source from a file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or decoded
    pub fn from_file(mixer: &Mixer, path: impl AsRef<Path>) -> Result<Self, AudioError> {
        let path = path.as_ref();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file = File::open(path).map_err(|e| AudioError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);
        let source = Decoder::new(reader).map_err(|e| AudioError::DecodeError(e.to_string()))?;

        let sink = Sink::connect_new(mixer);
        sink.append(source);
        sink.pause(); // Start paused

        Ok(Self {
            sink,
            state: PlaybackState::Stopped,
            looping: false,
            name,
        })
    }

    /// Create an audio source from bytes
    pub fn from_bytes(
        mixer: &Mixer,
        bytes: Arc<[u8]>,
        name: impl Into<String>,
    ) -> Result<Self, AudioError> {
        let cursor = std::io::Cursor::new(bytes);
        let source = Decoder::new(cursor).map_err(|e| AudioError::DecodeError(e.to_string()))?;

        let sink = Sink::connect_new(mixer);
        sink.append(source);
        sink.pause();

        Ok(Self {
            sink,
            state: PlaybackState::Stopped,
            looping: false,
            name: name.into(),
        })
    }

    /// Play the audio
    pub fn play(&mut self) {
        self.sink.play();
        self.state = PlaybackState::Playing;
    }

    /// Pause the audio
    pub fn pause(&mut self) {
        self.sink.pause();
        self.state = PlaybackState::Paused;
    }

    /// Stop the audio (cannot be resumed)
    pub fn stop(&mut self) {
        self.sink.stop();
        self.state = PlaybackState::Stopped;
    }

    /// Set the volume (0.0 = silent, 1.0 = normal, >1.0 = amplified)
    pub fn set_volume(&mut self, volume: f32) {
        self.sink.set_volume(volume.max(0.0));
    }

    /// Get the current volume
    #[must_use]
    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }

    /// Set playback speed (1.0 = normal)
    pub fn set_speed(&mut self, speed: f32) {
        self.sink.set_speed(speed.max(0.1));
    }

    /// Get the current playback speed
    #[must_use]
    pub fn speed(&self) -> f32 {
        self.sink.speed()
    }

    /// Check if the audio has finished playing
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.sink.empty()
    }

    /// Get the current playback state
    #[must_use]
    pub fn state(&self) -> PlaybackState {
        if self.sink.empty() && self.state == PlaybackState::Playing {
            PlaybackState::Stopped
        } else {
            self.state
        }
    }

    /// Get the source name
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Check if looping is enabled
    #[must_use]
    pub const fn is_looping(&self) -> bool {
        self.looping
    }
}

impl std::fmt::Debug for AudioSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioSource")
            .field("name", &self.name)
            .field("state", &self.state)
            .field("looping", &self.looping)
            .field("volume", &self.sink.volume())
            .finish()
    }
}

/// Errors that can occur during audio operations
#[derive(Debug, Clone)]
pub enum AudioError {
    /// IO error reading file
    IoError(String),
    /// Error decoding audio data
    DecodeError(String),
    /// Error during playback
    PlayError(String),
    /// No audio device available
    NoDevice,
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {e}"),
            Self::DecodeError(e) => write!(f, "Decode error: {e}"),
            Self::PlayError(e) => write!(f, "Playback error: {e}"),
            Self::NoDevice => write!(f, "No audio output device available"),
        }
    }
}

impl std::error::Error for AudioError {}
