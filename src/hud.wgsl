struct VertexInput {
    @location(0) position: vec3<f32>,
};

@vertex
fn vs_main(bounds: VertexInput) -> @builtin(position) vec4<f32> {
    return vec4(bounds.position, 1.0);
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@fragment
fn fs_main(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(1.0, 1.0, 1.0, 0.01);
}
