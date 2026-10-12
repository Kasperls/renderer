use std::process::id;

use cgmath::{prelude::*, Vector3};
use wgpu::util::DeviceExt;
use crate::{texture, resources::load_binary, room::{self, Room}};

const SCALE_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    0.5, 0.0, 0.0, 0.0,
    0.0, 0.5, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.0, 1.0
);

const TEX_TILE_SIZE: i32 = 16;
pub const BLOCK_SIZE: f32 = 1.0;
/// (width, height)
const ATLAS_SIZE: (usize,usize) = (24, 24);

#[derive(Debug)]
pub struct BlockBox {
    pub bounds: [Vector3<f32>; 2]
}

impl BlockBox {
    fn new(pos: Vector3<f32>) -> Self {
        let offset = BLOCK_SIZE * 0.5;
        Self { 
            bounds: [
                Vector3::new(pos.x - offset, pos.y - offset, pos.z - offset),
                Vector3::new(pos.x + offset, pos.y + offset, pos.z + offset),
            ]
        }
    }

    pub fn collide(&self, pos: Vector3<f32>) -> bool {
        let mut count: u8 = 0;
        if self.bounds[0].x < pos.x && pos.x < self.bounds[1].x {count += 1};
        if self.bounds[0].y < pos.y && pos.y < self.bounds[1].y {count += 1};
        if self.bounds[0].z < pos.z && pos.z < self.bounds[1].z {count += 1};

        if count == 3 {
            return true;
        }
        return false;
    }
}
// litt spagetti
pub fn get_block_box(pos: Vector3<f32>) -> BlockBox {
    BlockBox::new(pos)
}

/// Cube handler. Does not support other types of objects
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
        
        // obj_handler.read_position_data(MAP);
        // obj_handler.read_room_data(vec![room::Room::from_folder("C:/Users/kasp9/rust/renderer/res/maps/room1").unwrap()]);
        obj_handler.read_room_data(vec![room::Room::read_complete_csv("C:/Users/kaspe/rust/renderer/res/maps/small_square.txt").unwrap()]);
        obj_handler.create_instances();
        obj_handler
        
    }

    pub fn read_room_data(&mut self, rooms: Vec<Room>) {
        // for hver block: self.positions.push(vec3) v self.atlas_offsets.push(atl_pos(vec2))
        for room in rooms {
            for (y, layer) in room.layers.iter().enumerate() {
                for (z, row) in layer.iter().enumerate() {
                    for (x, block) in row.iter().enumerate(){
                        match block {
                            -1 => {
                                continue;
                            }
                            _ => {
                                self.positions.push(Vector3::new(
                                    x as f32 * BLOCK_SIZE, 
                                    y as f32 * BLOCK_SIZE, 
                                    z as f32 * BLOCK_SIZE,
                                ));
                                let alt_y = (block) / ATLAS_SIZE.0 as i32;
                                let alt_x = block - alt_y*ATLAS_SIZE.0 as i32;
                                self.atlas_offsets.push(atl_pos(1 + alt_x as i32, 1 + alt_y as i32))
                            }
                        }
                    }
                }
            }

            // for layer in 0..room.dim[0] {
            //     for line in 0..room.dim[1] {
            //         for col in 0..room.dim[2] {
            //             let layer_cost = room.data.len() / room.size[0];
            //             let idx = layer * layer_cost + line*room.size[2] + col;
            //             let block_id = room.data[idx];
            //             match block_id {
            //                 -1 => {
            //                     continue;
            //                 }
            //                 _ => {
            //                     self.positions.push(Vector3::new(
            //                         col as f32 * BLOCK_SIZE, 
            //                         layer as f32 * BLOCK_SIZE, 
            //                         line as f32 * BLOCK_SIZE,
            //                     ));
            //                     let alt_y = (block_id) / ATLAS_SIZE.0 as i32;
            //                     let alt_x = block_id - alt_y*ATLAS_SIZE.0 as i32;
            //                     self.atlas_offsets.push(atl_pos(1 + alt_x as i32, 1 + alt_y as i32))
            //                 }
            //             }
            //         }
            //     }
            // }
        }
        // todo!()
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

    pub fn load_texture_atlas(file_name: &str, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<TextureAtlas> {
        let data = load_binary(file_name).unwrap();
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
    return [x * TEX_TILE_SIZE, y * TEX_TILE_SIZE, (x-1) * TEX_TILE_SIZE, (y-1) * TEX_TILE_SIZE];
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
                (SCALE_MATRIX * BLOCK_SIZE)
            ).into(),
            normal: cgmath::Matrix3::from(self.rotation).into(),
            atlas_rect: self.atlas_rect,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct InstanceRaw {
    // position from matrix
    pub model: [[f32; 4]; 4],
    // normal from matrix
    pub normal: [[f32; 3]; 3],
    // atlas rect
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
    pub fn get_textured_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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