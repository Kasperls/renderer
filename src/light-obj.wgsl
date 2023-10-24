struct InstanceInput {
    @location(1) model_matrix_0: vec4<f32>,
    @location(2) model_matrix_1: vec4<f32>,
    @location(3) model_matrix_2: vec4<f32>,
    @location(4) model_matrix_3: vec4<f32>,
    @location(5) color: vec4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput
    ) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let world_position = model_matrix * vec4<f32>(model.position, 1.0);
    let clip_position = camera.view_proj * world_position;
    return VertexOutput(clip_position, instance.color);
}

@fragment
fn fs_main(fragment_input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(fragment_input.color);
}