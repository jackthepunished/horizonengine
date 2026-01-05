// Particle shader with billboard support

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec3<f32>,
    _padding: f32,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) lifetime: f32,
    @location(2) velocity: vec3<f32>,
    @location(3) age: f32,
    @location(4) color: vec4<f32>,
    @location(5) size: f32,
    @location(6) rotation: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
}

@vertex
fn vs_main(
    in: VertexInput,
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    // Quad vertex offsets (TriangleList: 6 vertices)
    let offsets = array<vec2<f32>, 6>(
        vec2<f32>(-0.5, -0.5),
        vec2<f32>(0.5, -0.5),
        vec2<f32>(0.5, 0.5),
        vec2<f32>(0.5, 0.5),
        vec2<f32>(-0.5, 0.5),
        vec2<f32>(-0.5, -0.5)
    );
    
    let uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 1.0)
    );

    let idx = vertex_index % 6u;
    let offset = offsets[idx];
    out.uv = uvs[idx];
    out.color = in.color;

    // Billboard calculation
    // Get camera right and up vectors from view_proj
    // (This is a simplification, but works for demo)
    let right = vec3<f32>(camera.view_proj[0][0], camera.view_proj[1][0], camera.view_proj[2][0]);
    let up = vec3<f32>(camera.view_proj[0][1], camera.view_proj[1][1], camera.view_proj[2][1]);
    
    // Scale by particle size
    let scale = in.size;
    
    // Rotate offset
    let s = sin(in.rotation);
    let c = cos(in.rotation);
    let rotated_offset = vec2<f32>(
        offset.x * c - offset.y * s,
        offset.x * s + offset.y * c
    );
    
    let world_pos = in.position + (right * rotated_offset.x * scale) + (up * rotated_offset.y * scale);
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple circular particle
    let dist = length(in.uv - vec2<f32>(0.5, 0.5));
    if dist > 0.5 {
        discard;
    }
    
    // Soft edges
    let alpha = 1.0 - smoothstep(0.4, 0.5, dist);
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
