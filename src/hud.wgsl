struct VertexInput {
    @location(0) position: vec3<f32>,
};

@vertex
fn vs_main(bounds: VertexInput) -> @builtin(position) vec4<f32> {
    return vec4(bounds.position, 1.0);
}

@group(0) @binding(0) var t_hud: texture_2d<f32>;
@group(0) @binding(1) var s_hud: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

fn transform_uv(uv: vec2<f32>, rect: vec4<f32>) -> vec2<f32> {
    var relative_rect = rect;
    var uv_trans: vec2<f32>;
    uv_trans = mix(relative_rect.xy, relative_rect.zw, uv);
    return uv_trans;
}

// fn get_pos(pos: vec3<f32>) -> vec2<f32> {
//     let rect = vec4(0.0, 0.0, 1.0/20.0, 1.0/20.0);
//     var offset = vec2(0.0, 1.0);
//     let uv_r = pos.xy ;
//     let uv = mix(rect.xy , rect.zw / 30.0, uv_r);
//     return uv * vec2(0.5, -0.5) + offset;
// }


fn get_pos(pos: vec3<f32>) -> vec2<f32> {
    let sides = 32.0;
    let rect = vec4(0.0, 0.0, 1.0 / sides, 1.0 * sides / sides / sides);
    let uv_r = vec2(pos.x, pos.y);
    let offset = -vec2(10.7 * 1.5, 6.43 * 1.5);
    let uv = mix(rect.yx, rect.zw, uv_r);
    return (uv * vec2(0.561, 0.561) + offset);
}

@fragment
fn fs_main(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_hud, s_hud, get_pos(vertex_output.position.xyz));

    return vec4(color);
}
