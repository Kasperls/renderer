// Vertex shader
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) rotation_matrix_0: vec4<f32>,
    @location(10) rotation_matrix_1: vec4<f32>,
    @location(11) rotation_matrix_2: vec4<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
};  

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var model_matrix: mat4x4<f32> = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var normal_matrix = mat3x3<f32>(
        instance.rotation_matrix_0.xyz,
        instance.rotation_matrix_1.xyz,
        instance.rotation_matrix_2.xyz,
    );
    var out: VertexOutput;
    // sRGB to linear
    // out.color = pow(model.color, vec3<f32>(2.4));
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    out.normal = normal_matrix * model.normal;
    return out;
}
//  Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

struct LightUniform {
    @location(0) ambient_color: vec4<f32>,
    @location(1) diffuse_color: vec4<f32>,
    @location(2) diffuse_pos: vec3<f32>,
};

@group(2) @binding(0)
var<uniform> light_uniform: LightUniform;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> { 
    var object_color: vec4<f32>;
    object_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    var light: vec4<f32>;
    light = light_uniform.ambient_color;

    // var light_dir: vec3<f32> = (camera.view_proj * in.model_matrix * vec4(normalize(light_uniform.diffuse_pos - in.clip_position.xyz), 1.0)).xyz;
    var light_dir: vec3<f32> = normalize(light_uniform.diffuse_pos);
    var diffuse: f32 = max(dot(in.normal, light_dir), 0.0);
    light += vec4<f32>(light_uniform.diffuse_color) * diffuse;

    var out = object_color * light;
    
    return vec4<f32>(out);
}