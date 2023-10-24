
const shadowDepthTextureSize: f32 = 7200.0;
// https://gpuweb.github.io/gpuweb/wgsl/#texturesamplecompare DOCS!!
// Vertex shader
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) rotation_matrix_0: vec4<f32>,
    @location(10) rotation_matrix_1: vec4<f32>,
    @location(11) rotation_matrix_2: vec4<f32>,
    @location(12) texture_rect: vec4<u32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0) var<uniform> camera: CameraUniform;

struct Globals {
    @location(0) global_mx: mat4x4<f32>,
    @location(1) num_lights: i32,
}
@group(3) @binding(0) var<uniform> globals: Globals;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>, //proj_pos
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>, //world_normal
    @location(4) world_pos: vec4<f32>, //world_pos
    @location(5) texture_rect: vec4<f32>,
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

    let world_pos = (model_matrix * vec4<f32>(model.position, 1.0));

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.texture_rect = vec4<f32>(instance.texture_rect);

    out.normal = normal_matrix * model.normal;
    out.world_pos = world_pos;
    out.clip_position = camera.view_proj * world_pos;

    let out1: VertexOutput = VertexOutput(
        out.clip_position,
        out.tex_coords,
        out.normal,
        out.world_pos,
        out.texture_rect,
    );
    return out1;
}

//  Fragment shader
@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

struct LightUniform {
    @location(0) color: vec3<f32>,
    @location(1) position: vec4<f32>,
    @location(2) direction: vec3<f32>,
    @location(3) light_matrix: mat4x4<f32>,
};
@group(2) @binding(0) var<storage, read>  light_uniforms: array<LightUniform>;
@group(2) @binding(1) var t_depth: texture_depth_2d_array;
@group(2) @binding(2) var s_depth: sampler_comparison;

fn fetch_shadow(homogeneous_coords: vec4<f32>, offset: vec2<f32>, index: i32) -> f32 {
    if (homogeneous_coords.w <= 0.0) {
            return 1.0;
    }
    
    let flip_correction = vec2<f32>(0.5, -0.5);

    let proj_cords = homogeneous_coords.xyz / homogeneous_coords.w;
    let uv = proj_cords.xy * flip_correction + vec2<f32>(0.5, 0.5);
    
    return textureSampleCompareLevel(t_depth, s_depth, uv + offset, index, proj_cords.z - 0.00001);
}

fn transform_uv(uv: vec2<f32>, rect: vec4<f32>) -> vec2<f32> {
    var relative_rect = rect / 16.0 / 24.0;
    var uv_trans: vec2<f32>;
    uv_trans = mix(relative_rect.xy, relative_rect.zw, uv);
    return uv_trans;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> { 
    var object_color: vec4<f32>;
    object_color = textureSample(t_diffuse, s_diffuse, transform_uv(vertex.tex_coords, vertex.texture_rect));
    
    let normal = normalize(vertex.normal);

    var spot_light = 0.05;
    var color = vec3(0.0);
    let oneOverShadowDepthTextureSize = 1.0 / shadowDepthTextureSize;

    let ambient = 0.005;
    	
    for (var i = 0; i < globals.num_lights; i += 1) {

        let obj_light_dir = normalize(light_uniforms[i].position.xyz - vertex.world_pos.xyz);
        let light_color = light_uniforms[i].color;
        let diffuse = max(dot(normal, obj_light_dir), 0.0);

        if dot(normalize(light_uniforms[i].direction), obj_light_dir) > -.95 {
            color += light_color.xyz * object_color.xyz * (spot_light * diffuse / 9.0 + ambient);
            continue;
        }
        for (var y = -1; y <= 1; y++) {
                for (var x = -1; x <= 1; x++) { 
                    let offset = vec2<f32>(vec2(x, y)) * oneOverShadowDepthTextureSize * 2.0;
                    spot_light += fetch_shadow(light_uniforms[i].light_matrix * vertex.world_pos, offset, i);

                }
            }
        spot_light /= 9.0;


        color += light_color.xyz * object_color.xyz * (spot_light * diffuse + ambient);
    }
    spot_light /= f32(globals.num_lights);

    let out = vec4<f32>(color.rgb, 1.0);
    
    return vec4<f32>(out);
}