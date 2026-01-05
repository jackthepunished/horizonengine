//! Multi-light system with different light types
//!
//! Supports point lights, directional lights, and spot lights.

use bytemuck::{Pod, Zeroable};
use glam::Vec3;

/// Maximum number of lights supported
pub const MAX_LIGHTS: usize = 16;

/// Type of light
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum LightType {
    /// Point light - emits in all directions from a point
    #[default]
    Point = 0,
    /// Directional light - parallel rays (like the sun)
    Directional = 1,
    /// Spot light - cone of light from a point
    Spot = 2,
}

/// GPU-compatible light data
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GpuLight {
    /// Position (for point/spot) or direction (for directional)
    pub position: [f32; 3],
    /// Light type (0 = point, 1 = directional, 2 = spot)
    pub light_type: u32,
    /// Light color
    pub color: [f32; 3],
    /// Intensity/strength
    pub intensity: f32,
    /// Direction (for spot/directional lights)
    pub direction: [f32; 3],
    /// Inner cone angle for spot lights (cosine)
    pub inner_cone_cos: f32,
    /// Outer cone angle for spot lights (cosine)
    pub outer_cone_cos: f32,
    /// Constant attenuation
    pub constant: f32,
    /// Linear attenuation
    pub linear: f32,
    /// Quadratic attenuation
    pub quadratic: f32,
}

impl Default for GpuLight {
    fn default() -> Self {
        Self {
            position: [0.0, 5.0, 0.0],
            light_type: LightType::Point as u32,
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            direction: [0.0, -1.0, 0.0],
            inner_cone_cos: 0.9, // ~25 degrees
            outer_cone_cos: 0.8, // ~37 degrees
            constant: 1.0,
            linear: 0.09,
            quadratic: 0.032,
        }
    }
}

/// Light storage for multiple lights
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct LightStorage {
    /// Ambient light color
    pub ambient: [f32; 3],
    /// Number of active lights
    pub num_lights: u32,
    /// Light array
    pub lights: [GpuLight; MAX_LIGHTS],
}

impl Default for LightStorage {
    fn default() -> Self {
        Self {
            ambient: [0.1, 0.1, 0.1],
            num_lights: 0,
            lights: [GpuLight::default(); MAX_LIGHTS],
        }
    }
}

/// High-level light configuration
#[derive(Debug, Clone)]
pub struct PointLight {
    /// World position
    pub position: Vec3,
    /// Light color
    pub color: Vec3,
    /// Intensity
    pub intensity: f32,
    /// Attenuation: constant, linear, quadratic
    pub attenuation: (f32, f32, f32),
}

impl PointLight {
    /// Create a new point light
    #[must_use]
    pub fn new(position: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            position,
            color,
            intensity,
            attenuation: (1.0, 0.09, 0.032),
        }
    }

    /// Set attenuation values
    #[must_use]
    pub fn with_attenuation(mut self, constant: f32, linear: f32, quadratic: f32) -> Self {
        self.attenuation = (constant, linear, quadratic);
        self
    }

    /// Convert to GPU light
    #[must_use]
    pub fn to_gpu(&self) -> GpuLight {
        GpuLight {
            position: self.position.into(),
            light_type: LightType::Point as u32,
            color: self.color.into(),
            intensity: self.intensity,
            direction: [0.0, -1.0, 0.0],
            inner_cone_cos: 0.9,
            outer_cone_cos: 0.8,
            constant: self.attenuation.0,
            linear: self.attenuation.1,
            quadratic: self.attenuation.2,
        }
    }
}

/// Directional light (like the sun)
#[derive(Debug, Clone)]
pub struct DirectionalLight {
    /// Light direction (normalized)
    pub direction: Vec3,
    /// Light color
    pub color: Vec3,
    /// Intensity
    pub intensity: f32,
}

impl DirectionalLight {
    /// Create a new directional light
    #[must_use]
    pub fn new(direction: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            direction: direction.normalize(),
            color,
            intensity,
        }
    }

    /// Convert to GPU light
    #[must_use]
    pub fn to_gpu(&self) -> GpuLight {
        GpuLight {
            position: [0.0; 3],
            light_type: LightType::Directional as u32,
            color: self.color.into(),
            intensity: self.intensity,
            direction: self.direction.into(),
            inner_cone_cos: 0.0,
            outer_cone_cos: 0.0,
            constant: 1.0,
            linear: 0.0,
            quadratic: 0.0,
        }
    }
}

/// Spot light
#[derive(Debug, Clone)]
pub struct SpotLight {
    /// World position
    pub position: Vec3,
    /// Light direction
    pub direction: Vec3,
    /// Light color
    pub color: Vec3,
    /// Intensity
    pub intensity: f32,
    /// Inner cone angle in radians
    pub inner_angle: f32,
    /// Outer cone angle in radians
    pub outer_angle: f32,
    /// Attenuation
    pub attenuation: (f32, f32, f32),
}

impl SpotLight {
    /// Create a new spot light
    #[must_use]
    pub fn new(position: Vec3, direction: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            position,
            direction: direction.normalize(),
            color,
            intensity,
            inner_angle: 25.0_f32.to_radians(),
            outer_angle: 35.0_f32.to_radians(),
            attenuation: (1.0, 0.09, 0.032),
        }
    }

    /// Set cone angles
    #[must_use]
    pub fn with_angles(mut self, inner_degrees: f32, outer_degrees: f32) -> Self {
        self.inner_angle = inner_degrees.to_radians();
        self.outer_angle = outer_degrees.to_radians();
        self
    }

    /// Convert to GPU light
    #[must_use]
    pub fn to_gpu(&self) -> GpuLight {
        GpuLight {
            position: self.position.into(),
            light_type: LightType::Spot as u32,
            color: self.color.into(),
            intensity: self.intensity,
            direction: self.direction.into(),
            inner_cone_cos: self.inner_angle.cos(),
            outer_cone_cos: self.outer_angle.cos(),
            constant: self.attenuation.0,
            linear: self.attenuation.1,
            quadratic: self.attenuation.2,
        }
    }
}

/// Light manager for handling multiple lights
#[derive(Debug, Default)]
pub struct LightManager {
    /// Point lights
    pub point_lights: Vec<PointLight>,
    /// Directional lights
    pub directional_lights: Vec<DirectionalLight>,
    /// Spot lights
    pub spot_lights: Vec<SpotLight>,
    /// Ambient color
    pub ambient: Vec3,
}

impl LightManager {
    /// Create a new light manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            point_lights: Vec::new(),
            directional_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient: Vec3::splat(0.1),
        }
    }

    /// Add a point light
    pub fn add_point_light(&mut self, light: PointLight) {
        self.point_lights.push(light);
    }

    /// Add a directional light
    pub fn add_directional_light(&mut self, light: DirectionalLight) {
        self.directional_lights.push(light);
    }

    /// Add a spot light
    pub fn add_spot_light(&mut self, light: SpotLight) {
        self.spot_lights.push(light);
    }

    /// Set ambient light color
    pub fn set_ambient(&mut self, color: Vec3) {
        self.ambient = color;
    }

    /// Clear all lights
    pub fn clear(&mut self) {
        self.point_lights.clear();
        self.directional_lights.clear();
        self.spot_lights.clear();
    }

    /// Get total number of lights
    #[must_use]
    pub fn light_count(&self) -> usize {
        self.point_lights.len() + self.directional_lights.len() + self.spot_lights.len()
    }

    /// Build GPU light storage from current lights
    #[must_use]
    pub fn build_storage(&self) -> LightStorage {
        let mut storage = LightStorage {
            ambient: self.ambient.into(),
            ..Default::default()
        };

        let mut idx = 0;

        // Add directional lights first (typically most important)
        for light in &self.directional_lights {
            if idx >= MAX_LIGHTS {
                break;
            }
            storage.lights[idx] = light.to_gpu();
            idx += 1;
        }

        // Add point lights
        for light in &self.point_lights {
            if idx >= MAX_LIGHTS {
                break;
            }
            storage.lights[idx] = light.to_gpu();
            idx += 1;
        }

        // Add spot lights
        for light in &self.spot_lights {
            if idx >= MAX_LIGHTS {
                break;
            }
            storage.lights[idx] = light.to_gpu();
            idx += 1;
        }

        storage.num_lights = idx as u32;
        storage
    }
}
