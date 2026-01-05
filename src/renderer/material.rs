//! Material system for meshes

use bytemuck::{Pod, Zeroable};
use glam::Vec3;

/// Material properties for rendering
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MaterialUniform {
    /// Base color (RGB)
    pub color: [f32; 3],
    /// Padding for alignment
    _padding1: f32,
    /// Specular strength
    pub specular: f32,
    /// Shininess factor
    pub shininess: f32,
    /// Whether to use texture (1.0) or solid color (0.0)
    pub use_texture: f32,
    /// Padding for alignment
    _padding2: f32,
}

impl MaterialUniform {
    /// Create a new material uniform
    pub fn new(color: Vec3, specular: f32, shininess: f32, use_texture: bool) -> Self {
        Self {
            color: color.into(),
            _padding1: 0.0,
            specular,
            shininess,
            use_texture: if use_texture { 1.0 } else { 0.0 },
            _padding2: 0.0,
        }
    }
}

impl Default for MaterialUniform {
    fn default() -> Self {
        Self::new(Vec3::new(0.8, 0.8, 0.8), 0.5, 32.0, false)
    }
}

/// Material definition
#[derive(Debug, Clone)]
pub struct Material {
    /// Base color
    pub color: Vec3,
    /// Specular reflectivity (0.0 - 1.0)
    pub specular: f32,
    /// Shininess exponent
    pub shininess: f32,
    /// Whether this material uses a texture
    pub use_texture: bool,
}

impl Material {
    /// Create a new material with a color
    pub fn new(color: Vec3) -> Self {
        Self {
            color,
            specular: 0.5,
            shininess: 32.0,
            use_texture: false,
        }
    }

    /// Create a diffuse material (no specular)
    pub fn diffuse(color: Vec3) -> Self {
        Self {
            color,
            specular: 0.0,
            shininess: 1.0,
            use_texture: false,
        }
    }

    /// Create a shiny material
    pub fn shiny(color: Vec3) -> Self {
        Self {
            color,
            specular: 1.0,
            shininess: 64.0,
            use_texture: false,
        }
    }

    /// Create a textured material with a tint color
    pub fn textured(tint: Vec3) -> Self {
        Self {
            color: tint,
            specular: 0.5,
            shininess: 32.0,
            use_texture: true,
        }
    }

    /// Create a textured material with white tint (no color modification)
    pub fn textured_default() -> Self {
        Self::textured(Vec3::ONE)
    }

    /// Red material
    pub fn red() -> Self {
        Self::new(Vec3::new(0.9, 0.2, 0.2))
    }

    /// Green material
    pub fn green() -> Self {
        Self::new(Vec3::new(0.2, 0.9, 0.2))
    }

    /// Blue material
    pub fn blue() -> Self {
        Self::new(Vec3::new(0.2, 0.2, 0.9))
    }

    /// White material
    pub fn white() -> Self {
        Self::new(Vec3::ONE)
    }

    /// Gray material
    pub fn gray() -> Self {
        Self::new(Vec3::splat(0.5))
    }

    /// Convert to uniform data
    pub fn to_uniform(&self) -> MaterialUniform {
        MaterialUniform::new(self.color, self.specular, self.shininess, self.use_texture)
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new(Vec3::new(0.8, 0.8, 0.8))
    }
}
