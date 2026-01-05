// Skybox shader with gradient procedural sky

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_position: vec3<f32>,
}

struct SkyboxUniform {
    view_proj: mat4x4<f32>,
}

struct GradientSkyUniform {
    zenith_color: vec3<f32>,
    _padding1: f32,
    horizon_color: vec3<f32>,
    _padding2: f32,
    ground_color: vec3<f32>,
    _padding3: f32,
}

@group(0) @binding(0) var<uniform> skybox: SkyboxUniform;
@group(0) @binding(1) var<uniform> gradient: GradientSkyUniform;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform position
    var pos = skybox.view_proj * vec4<f32>(in.position, 1.0);
    
    // Set z = w so skybox is always at far plane (depth = 1.0)
    out.clip_position = pos.xyww;
    out.local_position = in.position;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Normalize the direction
    let dir = normalize(in.local_position);
    
    // Get the vertical component for gradient
    let y = dir.y;
    
    // Calculate sky color based on vertical position
    var color: vec3<f32>;
    
    if y >= 0.0 {
        // Above horizon: blend zenith to horizon
        let t = pow(1.0 - y, 2.0);  // More horizon near the edge
        color = mix(gradient.zenith_color, gradient.horizon_color, t);
    } else {
        // Below horizon: blend horizon to ground
        let t = pow(-y, 0.5);  // Faster transition to ground
        color = mix(gradient.horizon_color, gradient.ground_color, t);
    }
    
    return vec4<f32>(color, 1.0);
}
