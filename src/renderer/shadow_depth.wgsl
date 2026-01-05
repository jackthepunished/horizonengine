// Shadow depth-only shader for shadow map generation
// This shader only outputs depth values, no color

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

struct ShadowUniform {
    light_space_matrix: mat4x4<f32>,
    bias: f32,
    _padding: vec3<f32>,
}

struct ModelUniform {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> shadow: ShadowUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    let world_position = model.model * vec4<f32>(in.position, 1.0);
    out.clip_position = shadow.light_space_matrix * world_position;
    
    return out;
}

// No fragment shader needed - we only care about depth
// wgpu will automatically write to depth buffer
