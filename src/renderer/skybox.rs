//! Skybox rendering
//!
//! Provides cubemap-based skybox for environment rendering.

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use wgpu::util::DeviceExt;

/// Skybox vertex (just position)
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct SkyboxVertex {
    pub position: [f32; 3],
}

/// Skybox uniform data
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct SkyboxUniform {
    /// View-projection matrix (without translation)
    pub view_proj: [[f32; 4]; 4],
}

/// Skybox renderer
pub struct Skybox {
    /// Cube vertex buffer
    pub vertex_buffer: wgpu::Buffer,
    /// Number of vertices
    pub vertex_count: u32,
    /// Uniform buffer
    pub uniform_buffer: wgpu::Buffer,
    /// Optional cubemap texture (if using texture-based skybox)
    pub cubemap: Option<wgpu::TextureView>,
    /// Cubemap sampler
    pub sampler: wgpu::Sampler,
}

impl Skybox {
    /// Create a new skybox
    #[must_use]
    pub fn new(device: &wgpu::Device) -> Self {
        // Cube vertices (inside-facing for skybox)
        let vertices: [SkyboxVertex; 36] = [
            // Front face
            SkyboxVertex {
                position: [-1.0, -1.0, 1.0],
            },
            SkyboxVertex {
                position: [1.0, 1.0, 1.0],
            },
            SkyboxVertex {
                position: [1.0, -1.0, 1.0],
            },
            SkyboxVertex {
                position: [-1.0, -1.0, 1.0],
            },
            SkyboxVertex {
                position: [-1.0, 1.0, 1.0],
            },
            SkyboxVertex {
                position: [1.0, 1.0, 1.0],
            },
            // Back face
            SkyboxVertex {
                position: [1.0, -1.0, -1.0],
            },
            SkyboxVertex {
                position: [-1.0, 1.0, -1.0],
            },
            SkyboxVertex {
                position: [-1.0, -1.0, -1.0],
            },
            SkyboxVertex {
                position: [1.0, -1.0, -1.0],
            },
            SkyboxVertex {
                position: [1.0, 1.0, -1.0],
            },
            SkyboxVertex {
                position: [-1.0, 1.0, -1.0],
            },
            // Top face
            SkyboxVertex {
                position: [-1.0, 1.0, 1.0],
            },
            SkyboxVertex {
                position: [1.0, 1.0, -1.0],
            },
            SkyboxVertex {
                position: [1.0, 1.0, 1.0],
            },
            SkyboxVertex {
                position: [-1.0, 1.0, 1.0],
            },
            SkyboxVertex {
                position: [-1.0, 1.0, -1.0],
            },
            SkyboxVertex {
                position: [1.0, 1.0, -1.0],
            },
            // Bottom face
            SkyboxVertex {
                position: [-1.0, -1.0, -1.0],
            },
            SkyboxVertex {
                position: [1.0, -1.0, 1.0],
            },
            SkyboxVertex {
                position: [1.0, -1.0, -1.0],
            },
            SkyboxVertex {
                position: [-1.0, -1.0, -1.0],
            },
            SkyboxVertex {
                position: [-1.0, -1.0, 1.0],
            },
            SkyboxVertex {
                position: [1.0, -1.0, 1.0],
            },
            // Right face
            SkyboxVertex {
                position: [1.0, -1.0, 1.0],
            },
            SkyboxVertex {
                position: [1.0, 1.0, -1.0],
            },
            SkyboxVertex {
                position: [1.0, -1.0, -1.0],
            },
            SkyboxVertex {
                position: [1.0, -1.0, 1.0],
            },
            SkyboxVertex {
                position: [1.0, 1.0, 1.0],
            },
            SkyboxVertex {
                position: [1.0, 1.0, -1.0],
            },
            // Left face
            SkyboxVertex {
                position: [-1.0, -1.0, -1.0],
            },
            SkyboxVertex {
                position: [-1.0, 1.0, 1.0],
            },
            SkyboxVertex {
                position: [-1.0, -1.0, 1.0],
            },
            SkyboxVertex {
                position: [-1.0, -1.0, -1.0],
            },
            SkyboxVertex {
                position: [-1.0, 1.0, -1.0],
            },
            SkyboxVertex {
                position: [-1.0, 1.0, 1.0],
            },
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("skybox_vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let uniform = SkyboxUniform {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("skybox_uniform"),
            contents: bytemuck::bytes_of(&uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("skybox_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            vertex_buffer,
            vertex_count: 36,
            uniform_buffer,
            cubemap: None,
            sampler,
        }
    }

    /// Update skybox with camera matrices
    pub fn update(&self, queue: &wgpu::Queue, view: Mat4, projection: Mat4) {
        // Remove translation from view matrix (skybox should stay centered on camera)
        let mut view_no_translation = view;
        view_no_translation.w_axis.x = 0.0;
        view_no_translation.w_axis.y = 0.0;
        view_no_translation.w_axis.z = 0.0;

        let view_proj = projection * view_no_translation;

        let uniform = SkyboxUniform {
            view_proj: view_proj.to_cols_array_2d(),
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    /// Get vertex buffer layout
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SkyboxVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }

    /// Get bind group layout for skybox
    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("skybox_bind_group_layout"),
            entries: &[
                // Uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Cubemap texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }
}

/// Gradient skybox colors (for procedural sky)
#[derive(Debug, Clone)]
pub struct GradientSky {
    /// Top/zenith color
    pub zenith_color: [f32; 3],
    /// Horizon color
    pub horizon_color: [f32; 3],
    /// Ground/nadir color
    pub ground_color: [f32; 3],
}

impl Default for GradientSky {
    fn default() -> Self {
        Self {
            zenith_color: [0.1, 0.2, 0.5],  // Deep blue
            horizon_color: [0.5, 0.7, 1.0], // Light blue
            ground_color: [0.3, 0.25, 0.2], // Brown/gray
        }
    }
}

/// GPU data for gradient sky
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GradientSkyUniform {
    pub zenith_color: [f32; 3],
    _padding1: f32,
    pub horizon_color: [f32; 3],
    _padding2: f32,
    pub ground_color: [f32; 3],
    _padding3: f32,
}

impl From<&GradientSky> for GradientSkyUniform {
    fn from(sky: &GradientSky) -> Self {
        Self {
            zenith_color: sky.zenith_color,
            _padding1: 0.0,
            horizon_color: sky.horizon_color,
            _padding2: 0.0,
            ground_color: sky.ground_color,
            _padding3: 0.0,
        }
    }
}
