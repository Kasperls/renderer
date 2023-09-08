use cgmath::{prelude::*, Vector3};
use wgpu::util::DeviceExt;
use winit::event::ScanCode;
use crate::{texture, resources::load_binary};

const SCALE_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    0.5, 0.0, 0.0, 0.0,
    0.0, 0.5, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.0, 1.0
);

const TILE_SIZE: i32 = 16;

const MAP: [[[u8; 9]; 8]; 8] = [
    [
        [2, 2, 2, 2, 2, 2, 2, 2, 1], 
        [2, 2, 2, 2, 2, 2, 2, 2, 1], 
        [2, 2, 2, 2, 2, 2, 2, 2, 1], 
        [2, 2, 2, 2, 2, 2, 2, 2, 1], 
        [2, 2, 2, 2, 2, 2, 2, 2, 1], 
        [2, 2, 2, 2, 2, 2, 2, 2, 1], 
        [2, 2, 2, 2, 2, 2, 2, 2, 1], 
        [2, 2, 2, 2, 2, 2, 2, 2, 1]
    ],[
        [1, 0, 0, 1, 1, 0, 0, 1, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [1, 0, 0, 1, 1, 0, 0, 1, 1]
    ],[
        [1, 0, 0, 1, 1, 0, 0, 1, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [1, 0, 0, 1, 1, 0, 0, 1, 1]
    ],[
        [1, 0, 0, 1, 1, 0, 0, 1, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [1, 0, 0, 1, 1, 0, 0, 1, 1]
    ],[
        [1, 0, 0, 1, 1, 0, 0, 1, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [1, 0, 0, 1, 1, 0, 0, 1, 1]
    ],[
        [1, 0, 0, 1, 1, 0, 0, 1, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [1, 0, 0, 1, 1, 0, 0, 1, 1]
    ],[
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1]
    ],[
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1], 
        [0, 0, 0, 0, 0, 0, 0, 0, 1]
    ],
    
];

const MAP1: [[[u8; 9]; 8]; 8] = [
    [
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],[
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],[
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],[
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
[0, 0, 0, 0, 0, 0, 0, 0, 0],
[0, 0, 0, 0, 0, 0, 0, 0, 0],
[0, 0, 0, 0, 0, 0, 0, 0, 0],
[0, 0, 0, 0, 0, 0, 0, 0, 0],
[0, 0, 0, 0, 0, 0, 0, 0, 0],
[0, 0, 0, 0, 0, 0, 0, 0, 0],
[0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],[
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],[
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],[
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
[0, 0, 0, 0, 0, 0, 0, 0, 0],
[0, 0, 0, 0, 0, 0, 0, 0, 0],
[0, 0, 0, 0, 0, 0, 0, 0, 0],
[0, 0, 0, 0, 0, 0, 0, 0, 0],
[0, 0, 0, 0, 0, 0, 0, 0, 0],
[0, 0, 0, 0, 0, 0, 0, 0, 0],
[0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],[
        [0, 0, 0, 0, 0, 0, 0, 0, 0], 
        [0, 0, 0, 0, 0, 0, 0, 0, 0], 
        [0, 0, 0, 0, 0, 0, 0, 0, 0], 
        [0, 0, 0, 0, 0, 0, 0, 0, 0], 
        [0, 0, 0, 0, 0, 0, 0, 0, 0], 
        [0, 0, 0, 0, 0, 0, 0, 0, 0], 
        [0, 0, 0, 0, 0, 0, 0, 0, 0], 
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],
    
];

pub struct ObjectHandler {
    pub object: Object,
    texture_atlas: TextureAtlas,
    instances: Vec<InstanceRaw>,
    pub positions: Vec<Vector3<f32>>,
    pub atlas_offsets: Vec<[i32; 4]>
}

impl ObjectHandler {
    pub fn new(object: Object, texture_atlas: TextureAtlas) -> Self {
        
        let mut obj_handler = Self { 
            object, 
            texture_atlas, 
            instances: vec![], 
            positions: vec![],
            atlas_offsets: vec![]
        };

        obj_handler.read_position_data(MAP);
        obj_handler.create_instances();
        obj_handler
        
    }

    pub fn create_instance_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&self.instances),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        )
    }

    fn create_instances(&mut self) {
        for (position, offsets) in self.positions.iter().zip(self.atlas_offsets.iter()) {
            self.instances.push(Instance {
                position: *position,
                rotation: cgmath::Quaternion::one(),
                atlas_rect: *offsets,
            }.to_raw());
        }
    }

    fn read_position_data(&mut self, object_array: [[[u8; 9]; 8]; 8]) {
        for x in 0..8 {
            for y in 0..8 {
                for z in 0..9 {
                    match object_array[x][y][z] {
                        1 => {
                            self.positions.push(Vector3::new(z as f32, x as f32, y as f32));
                            self.atlas_offsets.push(atl_pos(6, 6));
                        },
                        2 => {
                            self.positions.push(Vector3::new(z as f32, x as f32, y as f32));
                            self.atlas_offsets.push(atl_pos(6, 7));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub async fn load_texture_atlas(file_name: &str, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<TextureAtlas> {
        let data = load_binary("texture_atlas.png").await.unwrap();
        let texture = texture::Texture::from_bytes(device, queue, &data, file_name).expect("failed to create texture");

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            label: Some("texture_atlas_bind_group_layout"), 
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, 
                    visibility: wgpu::ShaderStages::FRAGMENT, 
                    ty: wgpu::BindingType::Texture { 
                        multisampled: false, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        sample_type: wgpu::TextureSampleType::Float { 
                            filterable: true 
                        }, 
                    }, 
                    count: None, 
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1, 
                    visibility: wgpu::ShaderStages::FRAGMENT, 
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), 
                    count: None,
                },
            ] 
        });

        let bind_group = device.create_bind_group( &wgpu::BindGroupDescriptor { 
            label: Some("texture_atlas_bind_group"), 
            layout: &bind_group_layout, 
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ] 
        });

        Ok(TextureAtlas {
            texture,
            bind_group,
        })
    }
}

fn atl_pos(x: i32, y: i32) -> [i32; 4] {
    return [x * TILE_SIZE, y * TILE_SIZE, (x-1) * TILE_SIZE, (y-1) * TILE_SIZE];
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

pub struct Object {
    pub meshes: Vec<Mesh>,
}

pub struct TextureAtlas {
    texture: texture::Texture,
    bind_group: wgpu::BindGroup,
}

pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub atlas_rect: [i32; 4],
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (
                cgmath::Matrix4::from_translation(self.position) * 
                cgmath::Matrix4::from(self.rotation) * 
                SCALE_MATRIX
            ).into(),
            normal: cgmath::Matrix3::from(self.rotation).into(),
            atlas_rect: self.atlas_rect,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
    pub atlas_rect: [i32; 4],
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 25]>() as wgpu::BufferAddress,
                    shader_location: 12,
                    format: wgpu::VertexFormat::Uint32x4,
                },
            ],
        }
    }
}

pub trait DrawObject<'a> {
    // fn new_draw_object_instanced(&mut self, object: &'a Object, instance_buffer: &'a wgpu::Buffer, camera_bind_group: &'a wgpu::BindGroup);
    // fn new_draw_mesh_instanced(&mut self, mesh: &'a Mesh, material: &'a Material, instances: std::ops::Range<u32>, camera_bind_group: &'a wgpu::BindGroup);
    fn draw_mesh_instanced(&mut self, mesh: &'a Mesh, instances: std::ops::Range<u32>, camera_bind_group: &'a wgpu::BindGroup);
    fn draw_object_instanced(&mut self, object_handler: &'a ObjectHandler, instance_buffer: &'a wgpu::Buffer, camera_bind_group: &'a wgpu::BindGroup);
    
}
impl <'a, 'b> DrawObject<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    // fn new_draw_mesh_instanced(&mut self, mesh: &'b Mesh, material: &'b Material, instances: std::ops::Range<u32>, camera_bind_group: &'b wgpu::BindGroup) {
    //     self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
    //     self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    //     self.set_bind_group(0, &material.bind_group, &[]);
    //     self.set_bind_group(1, camera_bind_group, &[]);
    //     self.draw_indexed(0..mesh.num_elements, 0, instances);
    // }
    // fn new_draw_object_instanced(&mut self, object: &'b Object, instance_buffer: &'b wgpu::Buffer, camera_bind_group: &'b wgpu::BindGroup) {
    //     // let instances = 0..object.meshes.len() as u32;
    //     let instances = instance_buffer.size() / std::mem::size_of::<InstanceRaw>() as u64;
    //     self.set_vertex_buffer(1, instance_buffer.slice(..));

    //     for mesh in &object.meshes {
    //         let material = &object.materials[mesh.material];
    //         self.new_draw_mesh_instanced(mesh, material, 0..instances as u32, camera_bind_group);
    //     }
    // }

    fn draw_mesh_instanced(&mut self, mesh: &'b Mesh, instances: std::ops::Range<u32>, camera_bind_group: &'b wgpu::BindGroup) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_object_instanced(&mut self, object_handler: &'b ObjectHandler, instance_buffer: &'b wgpu::Buffer, camera_bind_group: &'b wgpu::BindGroup) {
        let instances = instance_buffer.size() / std::mem::size_of::<InstanceRaw>() as u64;

        self.set_bind_group(0, &object_handler.texture_atlas.bind_group, &[]);
        self.set_vertex_buffer(1, instance_buffer.slice(..));

        for mesh in object_handler.object.meshes.iter() {
            self.draw_mesh_instanced(mesh, 0..instances as u32, camera_bind_group);
        }
    }
}

impl Object {
    pub fn get_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor { 
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0, 
                        visibility: wgpu::ShaderStages::FRAGMENT, 
                        ty: wgpu::BindingType::Texture { 
                            multisampled: false, 
                            view_dimension: wgpu::TextureViewDimension::D2, 
                            sample_type: wgpu::TextureSampleType::Float { 
                                filterable: true 
                            }, 
                        }, 
                        count: None, 
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1, 
                        visibility: wgpu::ShaderStages::FRAGMENT, 
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), 
                        count: None,
                    },
                    // normal map here if needed
                ],
                label: Some("object_bind_group_layout"), 
            }
        )
    }
}