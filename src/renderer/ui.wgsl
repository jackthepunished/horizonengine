// UI shader for 2D quads and text

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@group(0) @binding(0) var<uniform> screen_size: vec2<f32>;

struct RectInput {
    @location(0) position: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
}

@vertex
fn vs_main(
    in: RectInput,
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    // Quad vertex offsets (TriangleList: 6 vertices)
    let offsets = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 0.0)
    );

    let offset = offsets[vertex_index % 6u];
    
    // Pixel coordinates to NDC (-1 to 1)
    let pixel_pos = in.position + offset * in.size;
    let ndc_x = (pixel_pos.x / screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (pixel_pos.y / screen_size.y) * 2.0; // Y is inverted in screen space
    
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.color = in.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
