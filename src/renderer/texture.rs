//! Texture loading and GPU management
//!
//! Provides texture loading from files and GPU upload for rendering.

use image::GenericImageView;
use std::path::Path;
use wgpu::util::DeviceExt;

/// A GPU texture with its view and sampler
#[derive(Debug)]
pub struct Texture {
    /// The GPU texture
    pub texture: wgpu::Texture,
    /// Texture view for binding
    pub view: wgpu::TextureView,
    /// Sampler for texture filtering
    pub sampler: wgpu::Sampler,
    /// Texture dimensions
    pub size: wgpu::Extent3d,
}

impl Texture {
    /// Load a texture from a file path
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or decoded
    pub fn from_path(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: impl AsRef<Path>,
        label: Option<&str>,
    ) -> Result<Self, TextureError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|e| TextureError::IoError(e.to_string()))?;
        Self::from_bytes(device, queue, &bytes, label)
    }

    /// Load a texture from raw bytes (PNG, JPEG, etc.)
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes cannot be decoded as an image
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: Option<&str>,
    ) -> Result<Self, TextureError> {
        let img =
            image::load_from_memory(bytes).map_err(|e| TextureError::DecodeError(e.to_string()))?;
        Self::from_image(device, queue, &img, label)
    }

    /// Create a texture from a `DynamicImage`
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self, TextureError> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        Self::from_rgba(device, queue, &rgba, dimensions, label)
    }

    /// Create a texture from raw RGBA data
    pub fn from_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rgba: &[u8],
        dimensions: (u32, u32),
        label: Option<&str>,
    ) -> Result<Self, TextureError> {
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            rgba,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("texture_sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
            size,
        })
    }

    /// Create a 1x1 white texture (useful as default/placeholder)
    #[must_use]
    pub fn white(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self::from_rgba(
            device,
            queue,
            &[255, 255, 255, 255],
            (1, 1),
            Some("white_texture"),
        )
        .expect("Failed to create white texture")
    }

    /// Create a 1x1 colored texture
    #[must_use]
    pub fn solid_color(device: &wgpu::Device, queue: &wgpu::Queue, color: [u8; 4]) -> Self {
        Self::from_rgba(device, queue, &color, (1, 1), Some("solid_color_texture"))
            .expect("Failed to create solid color texture")
    }

    /// Get texture width
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.size.width
    }

    /// Get texture height
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.size.height
    }

    /// Create a bind group layout for textures
    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                // Texture view
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    /// Create a bind group for this texture
    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture_bind_group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }
}

/// Errors that can occur during texture loading
#[derive(Debug, Clone)]
pub enum TextureError {
    /// IO error reading file
    IoError(String),
    /// Error decoding image data
    DecodeError(String),
}

impl std::fmt::Display for TextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {e}"),
            Self::DecodeError(e) => write!(f, "Decode error: {e}"),
        }
    }
}

impl std::error::Error for TextureError {}
