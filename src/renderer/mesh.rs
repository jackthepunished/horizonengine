//! Mesh and vertex definitions

use bytemuck::{Pod, Zeroable};
use glam::Vec3;

/// Vertex with position, normal, and UV coordinates
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    /// Create a new vertex
    pub const fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Self {
        Self {
            position,
            normal,
            uv,
        }
    }

    /// Get the vertex buffer layout for wgpu
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Normal
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // UV
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

/// A 3D mesh with vertices and indices
#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    /// GPU vertex buffer (created when uploaded)
    pub(crate) vertex_buffer: Option<wgpu::Buffer>,
    /// GPU index buffer (created when uploaded)
    pub(crate) index_buffer: Option<wgpu::Buffer>,
}

impl Mesh {
    /// Create a new empty mesh
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            vertex_buffer: None,
            index_buffer: None,
        }
    }

    /// Create a mesh from vertices and indices
    pub fn from_data(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self {
            vertices,
            indices,
            vertex_buffer: None,
            index_buffer: None,
        }
    }

    /// Create a unit cube centered at origin
    pub fn cube() -> Self {
        let vertices = vec![
            // Front face
            Vertex::new([-0.5, -0.5, 0.5], [0.0, 0.0, 1.0], [0.0, 0.0]),
            Vertex::new([0.5, -0.5, 0.5], [0.0, 0.0, 1.0], [1.0, 0.0]),
            Vertex::new([0.5, 0.5, 0.5], [0.0, 0.0, 1.0], [1.0, 1.0]),
            Vertex::new([-0.5, 0.5, 0.5], [0.0, 0.0, 1.0], [0.0, 1.0]),
            // Back face
            Vertex::new([0.5, -0.5, -0.5], [0.0, 0.0, -1.0], [0.0, 0.0]),
            Vertex::new([-0.5, -0.5, -0.5], [0.0, 0.0, -1.0], [1.0, 0.0]),
            Vertex::new([-0.5, 0.5, -0.5], [0.0, 0.0, -1.0], [1.0, 1.0]),
            Vertex::new([0.5, 0.5, -0.5], [0.0, 0.0, -1.0], [0.0, 1.0]),
            // Top face
            Vertex::new([-0.5, 0.5, 0.5], [0.0, 1.0, 0.0], [0.0, 0.0]),
            Vertex::new([0.5, 0.5, 0.5], [0.0, 1.0, 0.0], [1.0, 0.0]),
            Vertex::new([0.5, 0.5, -0.5], [0.0, 1.0, 0.0], [1.0, 1.0]),
            Vertex::new([-0.5, 0.5, -0.5], [0.0, 1.0, 0.0], [0.0, 1.0]),
            // Bottom face
            Vertex::new([-0.5, -0.5, -0.5], [0.0, -1.0, 0.0], [0.0, 0.0]),
            Vertex::new([0.5, -0.5, -0.5], [0.0, -1.0, 0.0], [1.0, 0.0]),
            Vertex::new([0.5, -0.5, 0.5], [0.0, -1.0, 0.0], [1.0, 1.0]),
            Vertex::new([-0.5, -0.5, 0.5], [0.0, -1.0, 0.0], [0.0, 1.0]),
            // Right face
            Vertex::new([0.5, -0.5, 0.5], [1.0, 0.0, 0.0], [0.0, 0.0]),
            Vertex::new([0.5, -0.5, -0.5], [1.0, 0.0, 0.0], [1.0, 0.0]),
            Vertex::new([0.5, 0.5, -0.5], [1.0, 0.0, 0.0], [1.0, 1.0]),
            Vertex::new([0.5, 0.5, 0.5], [1.0, 0.0, 0.0], [0.0, 1.0]),
            // Left face
            Vertex::new([-0.5, -0.5, -0.5], [-1.0, 0.0, 0.0], [0.0, 0.0]),
            Vertex::new([-0.5, -0.5, 0.5], [-1.0, 0.0, 0.0], [1.0, 0.0]),
            Vertex::new([-0.5, 0.5, 0.5], [-1.0, 0.0, 0.0], [1.0, 1.0]),
            Vertex::new([-0.5, 0.5, -0.5], [-1.0, 0.0, 0.0], [0.0, 1.0]),
        ];

        let indices = vec![
            0, 1, 2, 2, 3, 0, // Front
            4, 5, 6, 6, 7, 4, // Back
            8, 9, 10, 10, 11, 8, // Top
            12, 13, 14, 14, 15, 12, // Bottom
            16, 17, 18, 18, 19, 16, // Right
            20, 21, 22, 22, 23, 20, // Left
        ];

        Self::from_data(vertices, indices)
    }

    /// Create a plane on the XZ axis
    pub fn plane(size: f32) -> Self {
        let half = size / 2.0;
        let vertices = vec![
            Vertex::new([-half, 0.0, half], [0.0, 1.0, 0.0], [0.0, 0.0]),
            Vertex::new([half, 0.0, half], [0.0, 1.0, 0.0], [1.0, 0.0]),
            Vertex::new([half, 0.0, -half], [0.0, 1.0, 0.0], [1.0, 1.0]),
            Vertex::new([-half, 0.0, -half], [0.0, 1.0, 0.0], [0.0, 1.0]),
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];

        Self::from_data(vertices, indices)
    }

    /// Create a UV sphere
    pub fn sphere(radius: f32, segments: u32, rings: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for ring in 0..=rings {
            let phi = std::f32::consts::PI * ring as f32 / rings as f32;
            let y = radius * phi.cos();
            let ring_radius = radius * phi.sin();

            for segment in 0..=segments {
                let theta = 2.0 * std::f32::consts::PI * segment as f32 / segments as f32;
                let x = ring_radius * theta.cos();
                let z = ring_radius * theta.sin();

                let position = Vec3::new(x, y, z);
                let normal = position.normalize();

                vertices.push(Vertex::new(
                    position.into(),
                    normal.into(),
                    [segment as f32 / segments as f32, ring as f32 / rings as f32],
                ));
            }
        }

        for ring in 0..rings {
            for segment in 0..segments {
                let current = ring * (segments + 1) + segment;
                let next = current + segments + 1;

                indices.push(current);
                indices.push(next);
                indices.push(current + 1);

                indices.push(current + 1);
                indices.push(next);
                indices.push(next + 1);
            }
        }

        Self::from_data(vertices, indices)
    }

    /// Get the number of indices
    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }

    /// Check if the mesh has been uploaded to GPU
    pub fn is_uploaded(&self) -> bool {
        self.vertex_buffer.is_some() && self.index_buffer.is_some()
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}
