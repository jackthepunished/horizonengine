//! Animation clips and keyframes
//!
//! Defines animation data structures for skeletal and property animation.

use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

/// Interpolation method for keyframes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Interpolation {
    /// Linear interpolation
    #[default]
    Linear,
    /// Step/discrete (no interpolation)
    Step,
    /// Cubic spline interpolation
    CubicSpline,
}

/// A single keyframe with timestamp and value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe<T> {
    /// Time in seconds
    pub time: f32,
    /// Value at this keyframe
    pub value: T,
    /// In-tangent for cubic spline (optional)
    pub in_tangent: Option<T>,
    /// Out-tangent for cubic spline (optional)
    pub out_tangent: Option<T>,
}

impl<T: Clone> Keyframe<T> {
    /// Create a new keyframe
    pub fn new(time: f32, value: T) -> Self {
        Self {
            time,
            value,
            in_tangent: None,
            out_tangent: None,
        }
    }

    /// Create a keyframe with cubic spline tangents
    pub fn with_tangents(time: f32, value: T, in_tangent: T, out_tangent: T) -> Self {
        Self {
            time,
            value,
            in_tangent: Some(in_tangent),
            out_tangent: Some(out_tangent),
        }
    }
}

/// Animation channel targeting a specific property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Channel {
    /// Translation keyframes
    Translation(Vec<Keyframe<Vec3>>),
    /// Rotation keyframes (as quaternions)
    Rotation(Vec<Keyframe<Quat>>),
    /// Scale keyframes
    Scale(Vec<Keyframe<Vec3>>),
    /// Morph target weights
    MorphWeights(Vec<Keyframe<Vec<f32>>>),
}

impl Channel {
    /// Get the duration of this channel
    #[must_use]
    pub fn duration(&self) -> f32 {
        match self {
            Self::Translation(keys) => keys.last().map_or(0.0, |k| k.time),
            Self::Rotation(keys) => keys.last().map_or(0.0, |k| k.time),
            Self::Scale(keys) => keys.last().map_or(0.0, |k| k.time),
            Self::MorphWeights(keys) => keys.last().map_or(0.0, |k| k.time),
        }
    }
}

/// A complete animation clip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationClip {
    /// Clip name
    pub name: String,
    /// Duration in seconds
    pub duration: f32,
    /// Interpolation method
    pub interpolation: Interpolation,
    /// Channels indexed by target (bone index or property name)
    pub channels: Vec<(usize, Channel)>,
}

impl AnimationClip {
    /// Create a new empty animation clip
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            duration: 0.0,
            interpolation: Interpolation::Linear,
            channels: Vec::new(),
        }
    }

    /// Add a channel to the clip
    pub fn add_channel(&mut self, target: usize, channel: Channel) {
        let channel_duration = channel.duration();
        self.channels.push((target, channel));
        self.duration = self.duration.max(channel_duration);
    }

    /// Sample translation at a given time
    #[must_use]
    pub fn sample_translation(&self, target: usize, time: f32) -> Option<Vec3> {
        for (t, channel) in &self.channels {
            if *t == target
                && let Channel::Translation(keys) = channel
            {
                return Some(sample_vec3(keys, time, self.interpolation));
            }
        }
        None
    }

    /// Sample rotation at a given time
    #[must_use]
    pub fn sample_rotation(&self, target: usize, time: f32) -> Option<Quat> {
        for (t, channel) in &self.channels {
            if *t == target
                && let Channel::Rotation(keys) = channel
            {
                return Some(sample_quat(keys, time, self.interpolation));
            }
        }
        None
    }

    /// Sample scale at a given time
    #[must_use]
    pub fn sample_scale(&self, target: usize, time: f32) -> Option<Vec3> {
        for (t, channel) in &self.channels {
            if *t == target
                && let Channel::Scale(keys) = channel
            {
                return Some(sample_vec3(keys, time, self.interpolation));
            }
        }
        None
    }
}

impl Default for AnimationClip {
    fn default() -> Self {
        Self::new("untitled")
    }
}

/// Sample Vec3 keyframes at a given time
fn sample_vec3(keyframes: &[Keyframe<Vec3>], time: f32, interp: Interpolation) -> Vec3 {
    if keyframes.is_empty() {
        return Vec3::ZERO;
    }

    if time <= keyframes[0].time {
        return keyframes[0].value;
    }

    if time >= keyframes.last().unwrap().time {
        return keyframes.last().unwrap().value;
    }

    // Find surrounding keyframes
    for i in 0..keyframes.len() - 1 {
        let k0 = &keyframes[i];
        let k1 = &keyframes[i + 1];

        if time >= k0.time && time < k1.time {
            let t = (time - k0.time) / (k1.time - k0.time);
            return match interp {
                Interpolation::Step => k0.value,
                Interpolation::Linear => k0.value.lerp(k1.value, t),
                Interpolation::CubicSpline => {
                    // Hermite spline interpolation
                    let dt = k1.time - k0.time;
                    let t2 = t * t;
                    let t3 = t2 * t;
                    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
                    let h10 = t3 - 2.0 * t2 + t;
                    let h01 = -2.0 * t3 + 3.0 * t2;
                    let h11 = t3 - t2;

                    let out_tan = k0.out_tangent.unwrap_or(Vec3::ZERO);
                    let in_tan = k1.in_tangent.unwrap_or(Vec3::ZERO);

                    k0.value * h00 + out_tan * dt * h10 + k1.value * h01 + in_tan * dt * h11
                }
            };
        }
    }

    keyframes.last().unwrap().value
}

/// Sample Quat keyframes at a given time
fn sample_quat(keyframes: &[Keyframe<Quat>], time: f32, interp: Interpolation) -> Quat {
    if keyframes.is_empty() {
        return Quat::IDENTITY;
    }

    if time <= keyframes[0].time {
        return keyframes[0].value;
    }

    if time >= keyframes.last().unwrap().time {
        return keyframes.last().unwrap().value;
    }

    // Find surrounding keyframes
    for i in 0..keyframes.len() - 1 {
        let k0 = &keyframes[i];
        let k1 = &keyframes[i + 1];

        if time >= k0.time && time < k1.time {
            let t = (time - k0.time) / (k1.time - k0.time);
            return match interp {
                Interpolation::Step => k0.value,
                Interpolation::Linear | Interpolation::CubicSpline => k0.value.slerp(k1.value, t),
            };
        }
    }

    keyframes.last().unwrap().value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_clip_sampling() {
        let mut clip = AnimationClip::new("test");

        let translation_keys = vec![
            Keyframe::new(0.0, Vec3::ZERO),
            Keyframe::new(1.0, Vec3::new(10.0, 0.0, 0.0)),
        ];

        clip.add_channel(0, Channel::Translation(translation_keys));

        assert_eq!(clip.duration, 1.0);

        // Sample at t=0.5, should be halfway
        let pos = clip.sample_translation(0, 0.5).unwrap();
        assert!((pos.x - 5.0).abs() < 0.01);
    }
}
