// Post-processing shader with tone mapping, bloom, and vignette

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct PostProcessUniform {
    bloom_intensity: f32,
    bloom_threshold: f32,
    exposure: f32,
    gamma: f32,
    vignette_intensity: f32,
    bloom_enabled: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0) var hdr_texture: texture_2d<f32>;
@group(0) @binding(1) var hdr_sampler: sampler;
@group(0) @binding(2) var<uniform> config: PostProcessUniform;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

// ACES Filmic Tone Mapping
fn aces_tonemap(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return saturate((x * (a * x + b)) / (x * (c * x + d) + e));
}

// Reinhard tone mapping (simpler alternative)
fn reinhard_tonemap(hdr: vec3<f32>) -> vec3<f32> {
    return hdr / (hdr + vec3<f32>(1.0));
}

// Vignette effect
fn apply_vignette(color: vec3<f32>, uv: vec2<f32>, intensity: f32) -> vec3<f32> {
    let center = uv - vec2<f32>(0.5);
    let dist = length(center);
    let vignette = 1.0 - smoothstep(0.3, 0.8, dist) * intensity;
    return color * vignette;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample HDR color
    var hdr_color = textureSample(hdr_texture, hdr_sampler, in.uv).rgb;
    
    // Apply exposure
    hdr_color = hdr_color * config.exposure;
    
    // Apply ACES tone mapping
    var ldr_color = aces_tonemap(hdr_color);
    
    // Apply vignette if enabled
    if config.vignette_intensity > 0.0 {
        ldr_color = apply_vignette(ldr_color, in.uv, config.vignette_intensity);
    }
    
    // Apply gamma correction
    let gamma_corrected = pow(ldr_color, vec3<f32>(1.0 / config.gamma));
    
    return vec4<f32>(gamma_corrected, 1.0);
}
