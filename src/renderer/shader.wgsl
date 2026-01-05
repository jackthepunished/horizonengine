// Basic 3D shader with lighting, material, and texture support

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec3<f32>,
    _padding: f32,
}

struct ModelUniform {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

struct LightUniform {
    position: vec3<f32>,
    _padding1: f32,
    color: vec3<f32>,
    _padding2: f32,
    ambient: vec3<f32>,
    _padding3: f32,
}

struct MaterialUniform {
    color: vec3<f32>,
    _padding1: f32,
    specular: f32,
    shininess: f32,
    use_texture: f32,  // 1.0 = use texture, 0.0 = use material color only
    _padding2: f32,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<uniform> light: LightUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var<uniform> material: MaterialUniform;

// Texture bindings
@group(2) @binding(1) var diffuse_texture: texture_2d<f32>;
@group(2) @binding(2) var diffuse_sampler: sampler;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position = model.model * vec4<f32>(in.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    out.world_normal = normalize((model.normal_matrix * vec4<f32>(in.normal, 0.0)).xyz);
    out.uv = in.uv;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture and blend with material color
    let tex_color = textureSample(diffuse_texture, diffuse_sampler, in.uv);
    
    // Mix between material color and texture based on use_texture flag
    let base_color = mix(material.color, tex_color.rgb * material.color, material.use_texture);
    
    let specular_strength = material.specular;
    let shininess = material.shininess;

    // Ambient
    let ambient = light.ambient * base_color;

    // Diffuse
    let light_dir = normalize(light.position - in.world_position);
    let diff = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse = diff * light.color * base_color;

    // Specular (Blinn-Phong)
    let view_dir = normalize(camera.view_pos - in.world_position);
    let halfway_dir = normalize(light_dir + view_dir);
    let spec = pow(max(dot(in.world_normal, halfway_dir), 0.0), shininess);
    let specular = specular_strength * spec * light.color;

    let result = ambient + diffuse + specular;
    
    // Preserve texture alpha
    let alpha = mix(1.0, tex_color.a, material.use_texture);

    return vec4<f32>(result, alpha);
}
