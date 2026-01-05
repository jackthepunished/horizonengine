//! Steering behaviors for AI movement
//!
//! Provides classic steering behaviors for autonomous agents.

use glam::Vec3;

/// Output from a steering behavior
#[derive(Debug, Clone, Copy, Default)]
pub struct SteeringOutput {
    /// Linear acceleration
    pub linear: Vec3,
    /// Angular acceleration (yaw)
    pub angular: f32,
}

impl SteeringOutput {
    /// Zero steering
    pub const ZERO: Self = Self {
        linear: Vec3::ZERO,
        angular: 0.0,
    };

    /// Combine with another steering output
    #[must_use]
    pub fn combine(self, other: Self) -> Self {
        Self {
            linear: self.linear + other.linear,
            angular: self.angular + other.angular,
        }
    }

    /// Scale the output
    #[must_use]
    pub fn scale(self, factor: f32) -> Self {
        Self {
            linear: self.linear * factor,
            angular: self.angular * factor,
        }
    }
}

/// Trait for steering behaviors
pub trait SteeringBehavior {
    /// Calculate steering based on agent state
    fn calculate(&self, position: Vec3, velocity: Vec3) -> SteeringOutput;
}

/// Seek behavior - move towards target
#[derive(Debug, Clone)]
pub struct Seek {
    /// Target position
    pub target: Vec3,
    /// Maximum acceleration
    pub max_acceleration: f32,
}

impl Seek {
    /// Create a new seek behavior
    #[must_use]
    pub fn new(target: Vec3, max_acceleration: f32) -> Self {
        Self {
            target,
            max_acceleration,
        }
    }
}

impl SteeringBehavior for Seek {
    fn calculate(&self, position: Vec3, _velocity: Vec3) -> SteeringOutput {
        let direction = (self.target - position).normalize_or_zero();
        SteeringOutput {
            linear: direction * self.max_acceleration,
            angular: 0.0,
        }
    }
}

/// Flee behavior - move away from target
#[derive(Debug, Clone)]
pub struct Flee {
    /// Target position to flee from
    pub target: Vec3,
    /// Maximum acceleration
    pub max_acceleration: f32,
}

impl Flee {
    /// Create a new flee behavior
    #[must_use]
    pub fn new(target: Vec3, max_acceleration: f32) -> Self {
        Self {
            target,
            max_acceleration,
        }
    }
}

impl SteeringBehavior for Flee {
    fn calculate(&self, position: Vec3, _velocity: Vec3) -> SteeringOutput {
        let direction = (position - self.target).normalize_or_zero();
        SteeringOutput {
            linear: direction * self.max_acceleration,
            angular: 0.0,
        }
    }
}

/// Arrive behavior - move towards target and slow down
#[derive(Debug, Clone)]
pub struct Arrive {
    /// Target position
    pub target: Vec3,
    /// Maximum acceleration
    pub max_acceleration: f32,
    /// Maximum speed
    pub max_speed: f32,
    /// Slowing distance
    pub slow_radius: f32,
    /// Stopping distance
    pub target_radius: f32,
}

impl Arrive {
    /// Create a new arrive behavior
    #[must_use]
    pub fn new(target: Vec3, max_acceleration: f32, max_speed: f32) -> Self {
        Self {
            target,
            max_acceleration,
            max_speed,
            slow_radius: 5.0,
            target_radius: 0.5,
        }
    }
}

impl SteeringBehavior for Arrive {
    fn calculate(&self, position: Vec3, velocity: Vec3) -> SteeringOutput {
        let to_target = self.target - position;
        let distance = to_target.length();

        if distance < self.target_radius {
            return SteeringOutput::ZERO;
        }

        let target_speed = if distance > self.slow_radius {
            self.max_speed
        } else {
            self.max_speed * distance / self.slow_radius
        };

        let target_velocity = to_target.normalize_or_zero() * target_speed;
        let acceleration = target_velocity - velocity;

        let accel_magnitude = acceleration.length();
        if accel_magnitude > self.max_acceleration {
            return SteeringOutput {
                linear: acceleration.normalize_or_zero() * self.max_acceleration,
                angular: 0.0,
            };
        }

        SteeringOutput {
            linear: acceleration,
            angular: 0.0,
        }
    }
}

/// Wander behavior - random movement
#[derive(Debug, Clone)]
pub struct Wander {
    /// Wander circle distance
    pub offset: f32,
    /// Wander circle radius
    pub radius: f32,
    /// Maximum angle change per update
    pub rate: f32,
    /// Maximum acceleration
    pub max_acceleration: f32,
    /// Current wander angle
    angle: f32,
}

impl Wander {
    /// Create a new wander behavior
    #[must_use]
    pub fn new(max_acceleration: f32) -> Self {
        Self {
            offset: 5.0,
            radius: 2.0,
            rate: 0.5,
            max_acceleration,
            angle: 0.0,
        }
    }

    /// Update wander angle (call each frame with random delta)
    pub fn update(&mut self, random_delta: f32) {
        self.angle += (random_delta - 0.5) * 2.0 * self.rate;
    }
}

impl SteeringBehavior for Wander {
    fn calculate(&self, position: Vec3, velocity: Vec3) -> SteeringOutput {
        // Project wander circle in front of agent
        let forward = velocity.normalize_or_zero();
        let center = position + forward * self.offset;

        // Calculate target on wander circle
        let target = center
            + Vec3::new(
                self.angle.cos() * self.radius,
                0.0,
                self.angle.sin() * self.radius,
            );

        let direction = (target - position).normalize_or_zero();
        SteeringOutput {
            linear: direction * self.max_acceleration,
            angular: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seek() {
        let seek = Seek::new(Vec3::new(10.0, 0.0, 0.0), 5.0);
        let output = seek.calculate(Vec3::ZERO, Vec3::ZERO);

        assert!(output.linear.x > 0.0);
        assert!((output.linear.length() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_flee() {
        let flee = Flee::new(Vec3::new(10.0, 0.0, 0.0), 5.0);
        let output = flee.calculate(Vec3::ZERO, Vec3::ZERO);

        assert!(output.linear.x < 0.0); // Flee in opposite direction
    }

    #[test]
    fn test_arrive_slowing() {
        let arrive = Arrive::new(Vec3::new(1.0, 0.0, 0.0), 5.0, 10.0);
        let output = arrive.calculate(Vec3::ZERO, Vec3::ZERO);

        // Should have some acceleration towards target
        assert!(output.linear.x > 0.0);
    }

    #[test]
    fn test_wander() {
        let mut wander = Wander::new(5.0);
        let output = wander.calculate(Vec3::ZERO, Vec3::X);

        // Should produce some steering
        assert!(output.linear.length() > 0.0);

        // Update angle
        wander.update(0.8);
        let output2 = wander.calculate(Vec3::ZERO, Vec3::X);

        // Direction should change after update
        assert!((output.linear - output2.linear).length() > 0.001);
    }

    #[test]
    fn test_steering_output_combine() {
        let a = SteeringOutput {
            linear: Vec3::X,
            angular: 1.0,
        };
        let b = SteeringOutput {
            linear: Vec3::Y,
            angular: 2.0,
        };

        let combined = a.combine(b);
        assert!((combined.linear - Vec3::new(1.0, 1.0, 0.0)).length() < 0.01);
        assert!((combined.angular - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_steering_output_scale() {
        let output = SteeringOutput {
            linear: Vec3::X * 2.0,
            angular: 4.0,
        };
        let scaled = output.scale(0.5);

        assert!((scaled.linear.x - 1.0).abs() < 0.01);
        assert!((scaled.angular - 2.0).abs() < 0.01);
    }
}
