struct VertexInput {
    @location(0) position: vec3<f32>,
};

@vertex
fn vs_main(bounds: VertexInput) -> @builtin(position) vec4<f32> {
    return vec4(bounds.position, 1.0);
}

struct WindowSize {
    @location(0) size: vec2<f32>,
};

@group(0) @binding(0) var t_hud: texture_2d<f32>;
@group(0) @binding(1) var s_hud: sampler;
@group(0) @binding(2) var<uniform> w_size: WindowSize;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

fn get_pos(pos: vec3<f32>) -> vec2<f32> {
    let rect = vec4(0.0, 0.0, 1.0 / w_size.size.y, 1.0 * w_size.size.x / w_size.size.y / w_size.size.y);
    let uv_r = vec2(pos.x, pos.y);
    let uv = mix(rect.yx, rect.zw, uv_r);
    return (uv * vec2(0.561, 0.561));
}

@fragment
fn fs_main(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    // ENABLE FOR BLUR
    // var color = vec3(0.0, 0.0, 0.0);
    // for (var y = -1; y <= 1; y++) {
    //     for (var x = -1; x <= 1; x++) { 
    //         let offset = vec2<f32>(vec2(x, y)) * 0.0;
    //     }
    // }
        
    let pos = get_pos(vertex_output.position.xyz);
    let color = textureSample(t_hud, s_hud, pos).xyz;

    return vec4(color, 1.0);
}