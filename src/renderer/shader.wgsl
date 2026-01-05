// Basic 3D shader with lighting

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

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var<uniform> light: LightUniform;

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
    // Material properties
    let object_color = vec3<f32>(0.8, 0.8, 0.8);
    let specular_strength = 0.5;
    let shininess = 32.0;

    // Ambient
    let ambient = light.ambient * object_color;

    // Diffuse
    let light_dir = normalize(light.position - in.world_position);
    let diff = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse = diff * light.color * object_color;

    // Specular (Blinn-Phong)
    let view_dir = normalize(camera.view_pos - in.world_position);
    let halfway_dir = normalize(light_dir + view_dir);
    let spec = pow(max(dot(in.world_normal, halfway_dir), 0.0), shininess);
    let specular = specular_strength * spec * light.color;

    let result = ambient + diffuse + specular;

    return vec4<f32>(result, 1.0);
}
