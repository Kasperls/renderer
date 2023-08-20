use cgmath::Vector3;

pub struct _ObjectHandler {
    pub object: Object,
    pub instance_buffer: wgpu::Buffer,
    instances: Vec<Instance>,
    pub positions: Vec<Vector3<f32>>,
}

pub struct Object {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    pub fn to_raw(&self) -> InstaceRaw {
        InstaceRaw {
            model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)).into(),
            normal: cgmath::Matrix3::from(self.rotation).into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstaceRaw {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
}

impl InstaceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstaceRaw>() as wgpu::BufferAddress,
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
            ],
        }
    }
}

use crate::{model::Model, model::Material, model::Mesh};

pub trait DrawObject<'a> {
    fn new_draw_object_instanced(&mut self, object: &'a Model, instance_buffer: &'a wgpu::Buffer, camera_bind_group: &'a wgpu::BindGroup);
    fn new_draw_mesh_instanced(&mut self, mesh: &'a Mesh, material: &'a Material, instances: std::ops::Range<u32>, camera_bind_group: &'a wgpu::BindGroup);
}
impl <'a, 'b> DrawObject<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn new_draw_mesh_instanced(&mut self, mesh: &'b Mesh, material: &'b Material, instances: std::ops::Range<u32>, camera_bind_group: &'b wgpu::BindGroup) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
    fn new_draw_object_instanced(&mut self, object: &'b Model, instance_buffer: &'b wgpu::Buffer, camera_bind_group: &'b wgpu::BindGroup) {
        // let instances = 0..object.meshes.len() as u32;
        let instances = instance_buffer.size() / std::mem::size_of::<InstaceRaw>() as u64;
        self.set_vertex_buffer(1, instance_buffer.slice(..));

        for mesh in &object.meshes {
            let material = &object.materials[mesh.material];
            self.new_draw_mesh_instanced(mesh, material, 0..instances as u32, camera_bind_group);
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