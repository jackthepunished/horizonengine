//! Animation playback control
//!
//! Provides animation player for controlling clip playback.

use super::clip::AnimationClip;

/// Playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackState {
    /// Animation is playing
    Playing,
    /// Animation is paused
    Paused,
    /// Animation has stopped
    #[default]
    Stopped,
}

/// Animation player for controlling playback
#[derive(Debug)]
pub struct AnimationPlayer {
    /// Currently playing clip
    clip: Option<AnimationClip>,
    /// Current playback time in seconds
    current_time: f32,
    /// Playback speed multiplier
    speed: f32,
    /// Whether to loop the animation
    looping: bool,
    /// Current playback state
    state: PlaybackState,
    /// Blend weight (for animation blending)
    weight: f32,
}

impl AnimationPlayer {
    /// Create a new animation player
    #[must_use]
    pub fn new() -> Self {
        Self {
            clip: None,
            current_time: 0.0,
            speed: 1.0,
            looping: true,
            state: PlaybackState::Stopped,
            weight: 1.0,
        }
    }

    /// Set the animation clip to play
    pub fn set_clip(&mut self, clip: AnimationClip) {
        self.clip = Some(clip);
        self.current_time = 0.0;
    }

    /// Start or resume playback
    pub fn play(&mut self) {
        if self.clip.is_some() {
            self.state = PlaybackState::Playing;
        }
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.state = PlaybackState::Paused;
    }

    /// Stop playback and reset to beginning
    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
        self.current_time = 0.0;
    }

    /// Seek to a specific time
    pub fn seek(&mut self, time: f32) {
        if let Some(clip) = &self.clip {
            self.current_time = time.clamp(0.0, clip.duration);
        }
    }

    /// Update playback (call each frame)
    pub fn update(&mut self, delta_time: f32) {
        if self.state != PlaybackState::Playing {
            return;
        }

        if let Some(clip) = &self.clip {
            // Avoid division/modulo by zero for empty clips
            if clip.duration <= 0.0 {
                self.current_time = 0.0;
                if !self.looping {
                    self.state = PlaybackState::Stopped;
                }
                return;
            }

            self.current_time += delta_time * self.speed;

            if self.current_time >= clip.duration {
                if self.looping {
                    self.current_time %= clip.duration;
                } else {
                    self.current_time = clip.duration;
                    self.state = PlaybackState::Stopped;
                }
            }

            if self.current_time < 0.0 {
                if self.looping {
                    self.current_time = clip.duration + self.current_time % clip.duration;
                } else {
                    self.current_time = 0.0;
                    self.state = PlaybackState::Stopped;
                }
            }
        }
    }

    /// Get current playback time
    #[must_use]
    pub const fn current_time(&self) -> f32 {
        self.current_time
    }

    /// Get playback state
    #[must_use]
    pub const fn state(&self) -> PlaybackState {
        self.state
    }

    /// Check if currently playing
    #[must_use]
    pub const fn is_playing(&self) -> bool {
        matches!(self.state, PlaybackState::Playing)
    }

    /// Get playback speed
    #[must_use]
    pub const fn speed(&self) -> f32 {
        self.speed
    }

    /// Set playback speed (can be negative for reverse)
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    /// Check if looping
    #[must_use]
    pub const fn is_looping(&self) -> bool {
        self.looping
    }

    /// Set looping
    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }

    /// Get blend weight
    #[must_use]
    pub const fn weight(&self) -> f32 {
        self.weight
    }

    /// Set blend weight (0.0 to 1.0)
    pub fn set_weight(&mut self, weight: f32) {
        self.weight = weight.clamp(0.0, 1.0);
    }

    /// Get the current clip (if any)
    #[must_use]
    pub fn clip(&self) -> Option<&AnimationClip> {
        self.clip.as_ref()
    }

    /// Get the normalized playback time (0.0 to 1.0)
    ///
    /// Useful for UI progress bars or syncing with other systems.
    /// Returns 0.0 if no clip is loaded or duration is zero.
    #[must_use]
    pub fn normalized_time(&self) -> f32 {
        if let Some(clip) = &self.clip
            && clip.duration > 0.0
        {
            return self.current_time / clip.duration;
        }
        0.0
    }
}

impl Default for AnimationPlayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::clip::{AnimationClip, Channel, Keyframe};
    use glam::Vec3;

    #[test]
    fn test_animation_player_playback() {
        let mut player = AnimationPlayer::new();

        let mut clip = AnimationClip::new("test");
        clip.add_channel(
            0,
            Channel::Translation(vec![
                Keyframe::new(0.0, Vec3::ZERO),
                Keyframe::new(1.0, Vec3::X),
            ]),
        );

        player.set_clip(clip);
        player.play();

        assert!(player.is_playing());
        assert_eq!(player.current_time(), 0.0);

        // Update by 0.5 seconds
        player.update(0.5);
        assert!((player.current_time() - 0.5).abs() < 0.01);

        // Update past end with looping
        player.update(0.8);
        assert!(player.current_time() < 0.5); // Should have looped
    }
}
