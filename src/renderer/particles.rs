//! Particle system for visual effects
//!
//! GPU-accelerated particles with configurable emitters.

use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};
use wgpu::util::DeviceExt;

/// A single particle
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Particle {
    /// World position
    pub position: [f32; 3],
    /// Remaining lifetime (seconds)
    pub lifetime: f32,
    /// Velocity
    pub velocity: [f32; 3],
    /// Age (seconds since spawn)
    pub age: f32,
    /// Color (RGBA)
    pub color: [f32; 4],
    /// Size
    pub size: f32,
    /// Rotation (radians)
    pub rotation: f32,
    /// Padding
    _padding: [f32; 2],
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            lifetime: 1.0,
            velocity: [0.0; 3],
            age: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
            size: 1.0,
            rotation: 0.0,
            _padding: [0.0; 2],
        }
    }
}

/// Particle emitter configuration
#[derive(Debug, Clone)]
pub struct EmitterConfig {
    /// Maximum number of particles
    pub max_particles: u32,
    /// Particles spawned per second
    pub spawn_rate: f32,
    /// Particle lifetime range (min, max)
    pub lifetime: (f32, f32),
    /// Initial velocity range
    pub velocity_min: Vec3,
    pub velocity_max: Vec3,
    /// Initial size range
    pub size: (f32, f32),
    /// Start color
    pub start_color: Vec4,
    /// End color (fade to)
    pub end_color: Vec4,
    /// Gravity
    pub gravity: Vec3,
    /// Whether to loop
    pub looping: bool,
}

impl Default for EmitterConfig {
    fn default() -> Self {
        Self {
            max_particles: 1000,
            spawn_rate: 100.0,
            lifetime: (1.0, 2.0),
            velocity_min: Vec3::new(-1.0, 1.0, -1.0),
            velocity_max: Vec3::new(1.0, 3.0, 1.0),
            size: (0.1, 0.3),
            start_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            end_color: Vec4::new(1.0, 1.0, 1.0, 0.0),
            gravity: Vec3::new(0.0, -9.8, 0.0),
            looping: true,
        }
    }
}

impl EmitterConfig {
    /// Set maximum particles
    #[must_use]
    pub const fn with_max_particles(mut self, max: u32) -> Self {
        self.max_particles = max;
        self
    }

    /// Set spawn rate (particles per second)
    #[must_use]
    pub const fn with_spawn_rate(mut self, rate: f32) -> Self {
        self.spawn_rate = rate;
        self
    }

    /// Set lifetime range
    #[must_use]
    pub const fn with_lifetime(mut self, min: f32, max: f32) -> Self {
        self.lifetime = (min, max);
        self
    }

    /// Set velocity range
    #[must_use]
    pub fn with_velocity(mut self, min: Vec3, max: Vec3) -> Self {
        self.velocity_min = min;
        self.velocity_max = max;
        self
    }

    /// Set size range
    #[must_use]
    pub const fn with_size(mut self, min: f32, max: f32) -> Self {
        self.size = (min, max);
        self
    }

    /// Set colors
    #[must_use]
    pub fn with_colors(mut self, start: Vec4, end: Vec4) -> Self {
        self.start_color = start;
        self.end_color = end;
        self
    }

    /// Set gravity
    #[must_use]
    pub fn with_gravity(mut self, gravity: Vec3) -> Self {
        self.gravity = gravity;
        self
    }

    /// Set looping
    #[must_use]
    pub const fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }
}

/// Particle emitter
#[derive(Debug)]
pub struct ParticleEmitter {
    /// Configuration
    pub config: EmitterConfig,
    /// World position
    pub position: Vec3,
    /// All particles
    particles: Vec<Particle>,
    /// Spawn accumulator
    spawn_accumulator: f32,
    /// Whether emitter is active
    active: bool,
    /// GPU buffer (if uploaded)
    buffer: Option<wgpu::Buffer>,
}

impl ParticleEmitter {
    /// Create a new emitter
    #[must_use]
    pub fn new(config: EmitterConfig) -> Self {
        Self {
            particles: Vec::with_capacity(config.max_particles as usize),
            config,
            position: Vec3::ZERO,
            spawn_accumulator: 0.0,
            active: true,
            buffer: None,
        }
    }

    /// Set emitter position
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    /// Start emitting
    pub fn start(&mut self) {
        self.active = true;
    }

    /// Stop emitting (particles continue to live)
    pub fn stop(&mut self) {
        self.active = false;
    }

    /// Clear all particles
    pub fn clear(&mut self) {
        self.particles.clear();
    }

    /// Get active particle count
    #[must_use]
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    /// Check if emitter is active
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }

    /// Update all particles
    pub fn update(&mut self, delta_time: f32) {
        // Update existing particles
        self.particles.retain_mut(|particle| {
            particle.age += delta_time;

            // Apply gravity
            let gravity = self.config.gravity;
            particle.velocity[0] += gravity.x * delta_time;
            particle.velocity[1] += gravity.y * delta_time;
            particle.velocity[2] += gravity.z * delta_time;

            // Update position
            particle.position[0] += particle.velocity[0] * delta_time;
            particle.position[1] += particle.velocity[1] * delta_time;
            particle.position[2] += particle.velocity[2] * delta_time;

            // Interpolate color based on age
            let t = particle.age / particle.lifetime;
            let start = self.config.start_color;
            let end = self.config.end_color;
            particle.color = [
                start.x + (end.x - start.x) * t,
                start.y + (end.y - start.y) * t,
                start.z + (end.z - start.z) * t,
                start.w + (end.w - start.w) * t,
            ];

            // Keep if still alive
            particle.age < particle.lifetime
        });

        // Spawn new particles
        if self.active {
            self.spawn_accumulator += self.config.spawn_rate * delta_time;

            while self.spawn_accumulator >= 1.0
                && self.particles.len() < self.config.max_particles as usize
            {
                self.spawn_particle();
                self.spawn_accumulator -= 1.0;
            }
        }
    }

    /// Spawn a single particle
    fn spawn_particle(&mut self) {
        use std::f32::consts::PI;

        let lifetime =
            self.config.lifetime.0 + rand_f32() * (self.config.lifetime.1 - self.config.lifetime.0);

        let velocity = Vec3::new(
            lerp(
                self.config.velocity_min.x,
                self.config.velocity_max.x,
                rand_f32(),
            ),
            lerp(
                self.config.velocity_min.y,
                self.config.velocity_max.y,
                rand_f32(),
            ),
            lerp(
                self.config.velocity_min.z,
                self.config.velocity_max.z,
                rand_f32(),
            ),
        );

        let size = lerp(self.config.size.0, self.config.size.1, rand_f32());

        let particle = Particle {
            position: self.position.into(),
            lifetime,
            velocity: velocity.into(),
            age: 0.0,
            color: self.config.start_color.into(),
            size,
            rotation: rand_f32() * PI * 2.0,
            _padding: [0.0; 2],
        };

        self.particles.push(particle);
    }

    /// Get particles for rendering
    #[must_use]
    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    /// Create or update GPU buffer
    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.particles.is_empty() {
            return;
        }

        let data = bytemuck::cast_slice(&self.particles);

        if let Some(buffer) = &self.buffer
            && buffer.size() >= data.len() as u64
        {
            queue.write_buffer(buffer, 0, data);
            return;
        }

        // Create new buffer
        self.buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("particle_buffer"),
                contents: data,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }),
        );
    }

    /// Get GPU buffer
    #[must_use]
    pub fn buffer(&self) -> Option<&wgpu::Buffer> {
        self.buffer.as_ref()
    }
}

/// Simple linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Simple pseudo-random (deterministic for testing)
fn rand_f32() -> f32 {
    use std::cell::Cell;
    thread_local! {
        static SEED: Cell<u32> = const { Cell::new(12345) };
    }

    SEED.with(|seed| {
        let mut s = seed.get();
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        seed.set(s);
        (s as f32) / (u32::MAX as f32)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_emitter_update() {
        let config = EmitterConfig {
            max_particles: 100,
            spawn_rate: 10.0,
            lifetime: (1.0, 1.0),
            ..Default::default()
        };

        let mut emitter = ParticleEmitter::new(config);

        // Update for 1 second
        emitter.update(1.0);

        // Should have spawned ~10 particles
        assert!(emitter.particle_count() >= 5);
        assert!(emitter.particle_count() <= 15);
    }

    #[test]
    fn test_particle_death() {
        let config = EmitterConfig {
            max_particles: 10,
            spawn_rate: 100.0,
            lifetime: (0.1, 0.1),
            ..Default::default()
        };

        let mut emitter = ParticleEmitter::new(config);

        // Spawn some
        emitter.update(0.05);
        let count = emitter.particle_count();
        assert!(count > 0);

        // Wait for them to die
        emitter.stop();
        emitter.update(0.2);

        assert_eq!(emitter.particle_count(), 0);
    }
}
