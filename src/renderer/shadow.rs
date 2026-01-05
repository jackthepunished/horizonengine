//! Shadow mapping system
//!
//! Implements shadow mapping for directional lights with depth-only pass.

use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

/// Shadow map configuration
#[derive(Debug, Clone)]
pub struct ShadowConfig {
    /// Shadow map resolution (width and height)
    pub resolution: u32,
    /// Near plane for shadow camera
    pub near: f32,
    /// Far plane for shadow camera
    pub far: f32,
    /// Size of the orthographic shadow frustum
    pub frustum_size: f32,
    /// Shadow bias to prevent shadow acne
    pub bias: f32,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            resolution: 2048,
            near: 0.1,
            far: 100.0,
            frustum_size: 20.0,
            bias: 0.005,
        }
    }
}

/// Shadow map for a single light
pub struct ShadowMap {
    /// Depth texture for shadow map
    pub texture: wgpu::Texture,
    /// Texture view for rendering
    pub depth_view: wgpu::TextureView,
    /// Sampler for shadow sampling
    pub sampler: wgpu::Sampler,
    /// Light space matrix (view-projection from light's perspective)
    pub light_space_matrix: Mat4,
    /// Configuration
    pub config: ShadowConfig,
    /// Uniform buffer for shadow data
    pub uniform_buffer: wgpu::Buffer,
}

/// Shadow uniform data for shader
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowUniform {
    /// Light space matrix
    pub light_space_matrix: [[f32; 4]; 4],
    /// Shadow bias
    pub bias: f32,
    /// Padding
    _padding: [f32; 3],
}

impl ShadowMap {
    /// Create a new shadow map
    #[must_use]
    pub fn new(device: &wgpu::Device, config: ShadowConfig) -> Self {
        let size = wgpu::Extent3d {
            width: config.resolution,
            height: config.resolution,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow_map_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::Less),
            ..Default::default()
        });

        let uniform = ShadowUniform {
            light_space_matrix: Mat4::IDENTITY.to_cols_array_2d(),
            bias: config.bias,
            _padding: [0.0; 3],
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("shadow_uniform_buffer"),
            contents: bytemuck::bytes_of(&uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            texture,
            depth_view,
            sampler,
            light_space_matrix: Mat4::IDENTITY,
            config,
            uniform_buffer,
        }
    }

    /// Update shadow map for a directional light
    pub fn update_for_directional_light(
        &mut self,
        queue: &wgpu::Queue,
        light_direction: Vec3,
        scene_center: Vec3,
    ) {
        // Calculate light position far from scene center
        let light_distance = self.config.far * 0.5;
        let light_pos = scene_center - light_direction.normalize() * light_distance;

        // Create orthographic projection for directional light
        let half_size = self.config.frustum_size;
        let projection = Mat4::orthographic_rh(
            -half_size,
            half_size,
            -half_size,
            half_size,
            self.config.near,
            self.config.far,
        );

        // Create view matrix looking at scene center
        let view = Mat4::look_at_rh(light_pos, scene_center, Vec3::Y);

        self.light_space_matrix = projection * view;

        // Update uniform buffer
        let uniform = ShadowUniform {
            light_space_matrix: self.light_space_matrix.to_cols_array_2d(),
            bias: self.config.bias,
            _padding: [0.0; 3],
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    /// Get bind group layout for shadow map sampling
    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow_bind_group_layout"),
            entries: &[
                // Shadow map texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Shadow sampler (comparison sampler)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                // Shadow uniform (light space matrix, bias)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    /// Create bind group for shadow sampling
    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow_bind_group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
            ],
        })
    }

    /// Get the resolution
    #[must_use]
    pub const fn resolution(&self) -> u32 {
        self.config.resolution
    }
}
