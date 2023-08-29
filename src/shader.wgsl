
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
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

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

    out.normal = normal_matrix * model.normal;
    out.world_pos = world_pos;
    out.clip_position = camera.view_proj * world_pos;
    // out.clip_position = light_uniform[1].light_matrix * world_pos;
    return out;
}

//  Fragment shader
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;


struct LightUniform {
    @location(0) color: vec4<f32>,
    @location(1) position: vec4<f32>,
    @location(2) light_matrix: mat4x4<f32>,
};
@group(2) @binding(0) var<storage, read>  light_uniform: array<LightUniform>;
@group(2) @binding(1) var t_depth: texture_depth_2d_array;
@group(2) @binding(2) var s_depth: sampler_comparison;

fn fetch_shadow(homogeneous_coords: vec4<f32>, offset: vec2<f32>, index: i32) -> f32 {
    if (homogeneous_coords.w <= 0.0) {
            return 1.0;
    }

    let flip_correction = vec2<f32>(0.5, -0.5);

    // let proj_correction = 1.0 / homogeneous_coords.w;
    let proj_cords = homogeneous_coords.xyz / homogeneous_coords.w;
    let uv = proj_cords.xy * flip_correction + vec2<f32>(0.5, 0.5);
    // let uv = homogeneous_coords.xy * flip_correction * proj_correction + vec2<f32>(0.5, 0.5);
    
    return textureSampleCompareLevel(t_depth, s_depth, uv + offset, index, proj_cords.z);
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> { 
    var object_color: vec4<f32>;
    object_color = textureSample(t_diffuse, s_diffuse, vertex.tex_coords);
    
    let normal = normalize(vertex.normal);

    var visibility = 0.05;
    var color = vec3(0.0);
    let oneOverShadowDepthTextureSize = 1.0 / shadowDepthTextureSize;

    let ambient = 0.01;
    	
    for (var i = 0; i < globals.num_lights; i += 1) {
        visibility += fetch_shadow(light_uniform[i].light_matrix * vertex.world_pos, vec2<f32>(0.0, 0.0), i);
    
        let light_dir = normalize(light_uniform[i].position.xyz - vertex.world_pos.xyz);
        let light_color = light_uniform[i].color;
        let diffuse = max(dot(normal, light_dir), 0.0);
        color += (diffuse + ambient) * light_color.xyz * object_color.xyz * visibility;
    }
    visibility /= f32(globals.num_lights);

    // for (var y = -1; y <= 1; y++) {
    //     for (var x = -1; x <= 1; x++) {
    //         let offset = vec2<f32>(vec2(x, y)) * oneOverShadowDepthTextureSize;

    //         visibility += fetch_shadow(light_uniform.light_matrix * vertex.world_pos, offset, index);
    //     }
    // }
    
    // visibility /= 9.0;


    let out = vec4<f32>(color.rgb, 1.0);
    
    return vec4<f32>(out);
}