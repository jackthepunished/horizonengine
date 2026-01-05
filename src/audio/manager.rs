//! Audio manager for managing audio output and sources

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use rodio::{OutputStream, OutputStreamBuilder, mixer::Mixer};

use super::source::{AudioError, AudioSource};

/// Manages audio output and all audio sources
pub struct AudioManager {
    /// The output stream (must be kept alive)
    _stream: OutputStream,
    /// The mixer for creating sinks
    mixer: Mixer,
    /// Named audio sources
    sources: HashMap<String, AudioSource>,
    /// Per-source volume settings (before master volume applied)
    source_volumes: HashMap<String, f32>,
    /// Master volume
    master_volume: f32,
    /// Whether audio is muted
    muted: bool,
}

impl AudioManager {
    /// Create a new audio manager
    ///
    /// # Errors
    ///
    /// Returns an error if no audio output device is available
    pub fn new() -> Result<Self, AudioError> {
        let stream = OutputStreamBuilder::from_default_device()
            .map_err(|_| AudioError::NoDevice)?
            .open_stream()
            .map_err(|_| AudioError::NoDevice)?;
        let mixer = stream.mixer().clone();

        Ok(Self {
            _stream: stream,
            mixer,
            sources: HashMap::new(),
            source_volumes: HashMap::new(),
            master_volume: 1.0,
            muted: false,
        })
    }

    /// Load an audio file and store it with a name
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be loaded
    pub fn load(
        &mut self,
        name: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<(), AudioError> {
        let name = name.into();
        let source = AudioSource::from_file(&self.mixer, path)?;
        self.sources.insert(name.clone(), source);
        self.source_volumes.insert(name, 1.0);
        Ok(())
    }

    /// Load audio from bytes
    pub fn load_bytes(
        &mut self,
        name: impl Into<String>,
        bytes: Arc<[u8]>,
    ) -> Result<(), AudioError> {
        let name = name.into();
        let source = AudioSource::from_bytes(&self.mixer, bytes, &name)?;
        self.sources.insert(name.clone(), source);
        self.source_volumes.insert(name, 1.0);
        Ok(())
    }

    /// Play an audio source by name
    pub fn play(&mut self, name: &str) -> bool {
        if let Some(source) = self.sources.get_mut(name) {
            if !self.muted {
                let source_vol = self.source_volumes.get(name).copied().unwrap_or(1.0);
                source.set_volume(source_vol * self.master_volume);
            }
            source.play();
            true
        } else {
            false
        }
    }

    /// Pause an audio source by name
    pub fn pause(&mut self, name: &str) -> bool {
        if let Some(source) = self.sources.get_mut(name) {
            source.pause();
            true
        } else {
            false
        }
    }

    /// Stop an audio source by name
    pub fn stop(&mut self, name: &str) -> bool {
        if let Some(source) = self.sources.get_mut(name) {
            source.stop();
            true
        } else {
            false
        }
    }

    /// Stop all audio sources
    pub fn stop_all(&mut self) {
        for source in self.sources.values_mut() {
            source.stop();
        }
    }

    /// Set volume for a specific source
    pub fn set_volume(&mut self, name: &str, volume: f32) -> bool {
        if let Some(source) = self.sources.get_mut(name) {
            let vol = volume.max(0.0);
            self.source_volumes.insert(name.to_string(), vol);
            let effective_volume = if self.muted {
                0.0
            } else {
                vol * self.master_volume
            };
            source.set_volume(effective_volume);
            true
        } else {
            false
        }
    }

    /// Set the master volume (affects all sources)
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.max(0.0);
        if !self.muted {
            for (name, source) in &mut self.sources {
                let source_vol = self.source_volumes.get(name).copied().unwrap_or(1.0);
                source.set_volume(source_vol * self.master_volume);
            }
        }
    }

    /// Get the master volume
    #[must_use]
    pub const fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Mute all audio
    pub fn mute(&mut self) {
        self.muted = true;
        for source in self.sources.values_mut() {
            source.set_volume(0.0);
        }
    }

    /// Unmute all audio
    pub fn unmute(&mut self) {
        self.muted = false;
        for (name, source) in &mut self.sources {
            let source_vol = self.source_volumes.get(name).copied().unwrap_or(1.0);
            source.set_volume(source_vol * self.master_volume);
        }
    }

    /// Toggle mute state
    pub fn toggle_mute(&mut self) {
        if self.muted {
            self.unmute();
        } else {
            self.mute();
        }
    }

    /// Check if audio is muted
    #[must_use]
    pub const fn is_muted(&self) -> bool {
        self.muted
    }

    /// Get a reference to an audio source
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&AudioSource> {
        self.sources.get(name)
    }

    /// Get a mutable reference to an audio source
    pub fn get_mut(&mut self, name: &str) -> Option<&mut AudioSource> {
        self.sources.get_mut(name)
    }

    /// Remove an audio source
    pub fn remove(&mut self, name: &str) -> Option<AudioSource> {
        self.source_volumes.remove(name);
        self.sources.remove(name)
    }

    /// Get the number of loaded sources
    #[must_use]
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// Get the mixer for creating custom sources
    #[must_use]
    pub fn mixer(&self) -> &Mixer {
        &self.mixer
    }

    /// Clean up finished one-shot sounds
    pub fn cleanup_finished(&mut self) {
        self.sources.retain(|_, source| !source.is_finished());
    }
}

impl std::fmt::Debug for AudioManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioManager")
            .field("source_count", &self.sources.len())
            .field("master_volume", &self.master_volume)
            .field("muted", &self.muted)
            .finish()
    }
}
