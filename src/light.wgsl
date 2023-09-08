

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) rotation_matrix_0: vec4<f32>,
    @location(10) rotation_matrix_1: vec4<f32>,
    @location(11) rotation_matrix_2: vec4<f32>,
};

struct LightUniform {
    @location(0) color: vec4<f32>,
    @location(1) position: vec4<f32>,
    @location(2) light_matrix: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> light_uniform: LightUniform;

struct Globals {
    @location(0) global_mx: mat4x4<f32>,
    @location(1) num_lights: u32,
}
@group(1) @binding(0) var<uniform> globals: Globals;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};  

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> @builtin(position) vec4<f32> {

    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let rotation_matrix = mat4x4<f32>(
        instance.rotation_matrix_0,
        instance.rotation_matrix_1,
        instance.rotation_matrix_2,
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );
   
    return light_uniform.light_matrix * model_matrix * vec4<f32>(model.position, 1.0);
}