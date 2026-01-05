//! Rendering module
//!
//! 3D rendering with wgpu

mod camera;
mod context;
mod lights;
mod material;
mod mesh;
mod particles;
mod postprocess;
mod shadow;
mod skybox;
mod texture;

pub use camera::Camera;
pub use context::{Light, ModelUniform, RenderFrame, Renderer, UiRect};
pub use lights::{DirectionalLight, GpuLight, LightManager, LightStorage, PointLight, SpotLight};
pub use material::{Material, MaterialUniform};
pub use mesh::{Mesh, Vertex};
pub use particles::{EmitterConfig, Particle, ParticleEmitter};
pub use postprocess::{FullscreenQuad, PostProcessConfig, PostProcessUniform, RenderTarget};
pub use shadow::{ShadowConfig, ShadowMap, ShadowUniform};
pub use skybox::{GradientSky, GradientSkyUniform, Skybox, SkyboxUniform};
pub use texture::{Texture, TextureError};
