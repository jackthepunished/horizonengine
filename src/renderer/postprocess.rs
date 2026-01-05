//! Post-processing effects
//!
//! Includes tone mapping, bloom, and other screen-space effects.

use bytemuck::{Pod, Zeroable};

/// Post-processing configuration
#[derive(Debug, Clone)]
pub struct PostProcessConfig {
    /// Enable bloom effect
    pub bloom_enabled: bool,
    /// Bloom intensity (0.0 - 2.0)
    pub bloom_intensity: f32,
    /// Bloom threshold (pixels brighter than this will bloom)
    pub bloom_threshold: f32,
    /// Exposure for tone mapping
    pub exposure: f32,
    /// Gamma correction value
    pub gamma: f32,
    /// Enable vignette
    pub vignette_enabled: bool,
    /// Vignette intensity
    pub vignette_intensity: f32,
}

impl Default for PostProcessConfig {
    fn default() -> Self {
        Self {
            bloom_enabled: true,
            bloom_intensity: 0.5,
            bloom_threshold: 1.0,
            exposure: 1.0,
            gamma: 2.2,
            vignette_enabled: false,
            vignette_intensity: 0.3,
        }
    }
}

/// GPU-compatible post-process uniform
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct PostProcessUniform {
    /// Bloom intensity
    pub bloom_intensity: f32,
    /// Bloom threshold
    pub bloom_threshold: f32,
    /// Exposure
    pub exposure: f32,
    /// Gamma
    pub gamma: f32,
    /// Vignette intensity (0 = disabled)
    pub vignette_intensity: f32,
    /// Bloom enabled flag
    pub bloom_enabled: f32,
    /// Padding
    _padding: [f32; 2],
}

impl PostProcessUniform {
    /// Create from config
    #[must_use]
    pub fn from_config(config: &PostProcessConfig) -> Self {
        Self {
            bloom_intensity: config.bloom_intensity,
            bloom_threshold: config.bloom_threshold,
            exposure: config.exposure,
            gamma: config.gamma,
            vignette_intensity: if config.vignette_enabled {
                config.vignette_intensity
            } else {
                0.0
            },
            bloom_enabled: if config.bloom_enabled { 1.0 } else { 0.0 },
            _padding: [0.0; 2],
        }
    }
}

impl Default for PostProcessUniform {
    fn default() -> Self {
        Self::from_config(&PostProcessConfig::default())
    }
}

/// Render target for HDR rendering and post-processing
pub struct RenderTarget {
    /// HDR color texture
    pub color_texture: wgpu::Texture,
    /// Color texture view
    pub color_view: wgpu::TextureView,
    /// Depth texture
    pub depth_texture: wgpu::Texture,
    /// Depth texture view
    pub depth_view: wgpu::TextureView,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
}

impl RenderTarget {
    /// Create a new render target
    #[must_use]
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        // HDR color texture (RGBA16Float for HDR values)
        let color_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("hdr_color_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Depth texture
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            color_texture,
            color_view,
            depth_texture,
            depth_view,
            width,
            height,
        }
    }

    /// Resize the render target
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        *self = Self::new(device, width, height);
    }
}

/// Full-screen quad for post-processing
pub struct FullscreenQuad {
    /// Vertex buffer
    pub vertex_buffer: wgpu::Buffer,
    /// Number of vertices
    pub vertex_count: u32,
}

/// Fullscreen quad vertex
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct QuadVertex {
    /// Position
    pub position: [f32; 2],
    /// UV coordinates
    pub uv: [f32; 2],
}

impl FullscreenQuad {
    /// Create a fullscreen quad
    #[must_use]
    pub fn new(device: &wgpu::Device) -> Self {
        use wgpu::util::DeviceExt;

        // Two triangles covering the full screen
        let vertices: [QuadVertex; 6] = [
            // First triangle
            QuadVertex {
                position: [-1.0, -1.0],
                uv: [0.0, 1.0],
            },
            QuadVertex {
                position: [1.0, -1.0],
                uv: [1.0, 1.0],
            },
            QuadVertex {
                position: [1.0, 1.0],
                uv: [1.0, 0.0],
            },
            // Second triangle
            QuadVertex {
                position: [-1.0, -1.0],
                uv: [0.0, 1.0],
            },
            QuadVertex {
                position: [1.0, 1.0],
                uv: [1.0, 0.0],
            },
            QuadVertex {
                position: [-1.0, 1.0],
                uv: [0.0, 0.0],
            },
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fullscreen_quad_vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            vertex_buffer,
            vertex_count: 6,
        }
    }

    /// Get vertex buffer layout
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
