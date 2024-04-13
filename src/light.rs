use crate::{object::Mesh, bindgroup::BindGroupContainer, pipelines::LightObjectPipeline};

pub struct LightContainer {
    pub index: usize,
    pub light: LightUniform,
    pub light_buffer: wgpu::Buffer,
    pub light_uniform_bind_group: wgpu::BindGroup,
    pub light_pass: wgpu::RenderPipeline,
    pub light_depth_texture_view: wgpu::TextureView,
}

impl LightContainer {
    fn new(
        index: usize,
        light: LightUniform, 
        light_buffer: wgpu::Buffer, 
        light_uniform_bind_group: wgpu::BindGroup, 
        light_pass: wgpu::RenderPipeline, 
        light_depth_texture_view: wgpu::TextureView,
    ) -> Self {
        Self {
            index,
            light,
            light_buffer,
            light_uniform_bind_group,
            light_pass,
            light_depth_texture_view,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LightUniform {
    pub color: Vector3<f32>,
    pub pos: cgmath::Point3<f32>,
    pub dir: cgmath::Vector3<f32>,
}

impl LightUniform {
    pub fn new(color: Vector3<f32>, pos: Point3<f32>, dir: Vector3<f32>) -> Self {
        Self {
            color,
            pos,
            dir,
        }
    }

    pub fn to_raw(&self) -> LightUniformRaw {
        LightUniformRaw {
            color: self.color.extend(1.0).into(),
            pos: self.pos.to_homogeneous().into(),
            dir: self.dir.extend(1.0).into(),
            view_proj: {
                let light_view = cgmath::Matrix4::look_at_rh(
                    self.pos,
                    self.pos + self.dir * 10.0,
                    Vector3::unit_y(),
                );
                let light_proj = cgmath::perspective(
                    Rad(std::f32::consts::PI/2.), 
                    1.0, 
                    0.1, 
                    10.0
                );
                let view_proj = crate::OPENGL_TO_WGPU_MATRIX * light_proj * light_view;
                view_proj.into()

            },
        }
    }

    pub fn get_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor { 
                label: Some("light_bind_group_layout"), 
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ] 
            }
        )
    }

    pub fn create_bind_group(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        device.create_bind_group(
            &wgpu::BindGroupDescriptor { 
                label: Some("light_bind_group"), 
                layout, 
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }] 
            }
        )
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniformRaw {
    pub color: [f32; 4],
    pub pos: [f32; 4],
    pub dir: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
}

use cgmath::{Rad, Vector3, Point3, EuclideanSpace, Vector4, Angle};
use wgpu::{util::DeviceExt, Color};

use crate::object::{Object, InstanceRaw};

pub trait LoadObjectVertexData<'a> {
    fn load_mesh_instanced(&mut self, mesh: &'a Mesh, instances: std::ops::Range<u32>);
    fn load_object_instanced(&mut self, object: &'a Object, instance_buffer: &'a wgpu::Buffer);
}
impl <'a, 'b> LoadObjectVertexData<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn load_mesh_instanced(&mut self, mesh: &'b Mesh, instances: std::ops::Range<u32>, ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
    fn load_object_instanced(&mut self, object: &'b Object, instance_buffer: &'b wgpu::Buffer) {
        // let instances = 0..object.meshes.len() as u32;
        let instances = instance_buffer.size() / std::mem::size_of::<InstanceRaw>() as u64;
        self.set_vertex_buffer(1, instance_buffer.slice(..));

        for mesh in &object.meshes {
            self.load_mesh_instanced(mesh, 0..instances as u32);
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightVertex {
    pub position: [f32; 3],
}

impl LightVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<LightVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ]
        }
    }
}

pub trait DrawLightVertexData<'a> {
    fn draw_light_mesh_instanced(&mut self, mesh: &'a Mesh, instances: std::ops::Range<u32>);
    fn draw_light_object_instanced(&mut self, object: &'a Object, instance_buffer: &'a wgpu::Buffer);
}
impl <'a, 'b> DrawLightVertexData<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_light_mesh_instanced(&mut self, mesh: &'b Mesh, instances: std::ops::Range<u32>, ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
    fn draw_light_object_instanced(&mut self, object: &'b Object, instance_buffer: &'b wgpu::Buffer) {
        let instances = instance_buffer.size() / std::mem::size_of::<LightInstanceRaw>() as u64;
        self.set_vertex_buffer(1, instance_buffer.slice(..));
        for mesh in &object.meshes {
            self.draw_light_mesh_instanced(mesh, 0..instances as u32);
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[derive(Debug)]
pub struct LightInstanceRaw {
    pub position: [[f32; 4]; 4],
    pub color: [f32; 3],
}

impl LightInstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<LightInstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

struct LightInstance {
    pub position: Point3<f32>,
    pub color: Vector3<f32>,
}

impl LightInstance {
    fn to_raw(&self) -> LightInstanceRaw {
        LightInstanceRaw {
            position: (
                cgmath::Matrix4::from_translation(self.position.to_vec()) *
                cgmath::Matrix4::from_scale(0.2)
            ).into(),
            color: self.color.into(),
        }
    }
}

pub struct LightHandler {
    pub object: Object,
    pub instances: Vec<LightInstanceRaw>,
    pub positions: Vec<Point3<f32>>,
    pub colors: Vec<Vector3<f32>>,
    pub pipeline: LightObjectPipeline,
}

impl LightHandler {
    pub fn new(object: Object, pipeline: LightObjectPipeline) -> Self {
        Self { 
            object, 
            instances: vec![], 
            positions: vec![], 
            colors: vec![], 
            pipeline, 
        }
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

    pub fn create_instances(&mut self) {
        for (position, color) in self.positions.iter().zip(self.colors.iter()) {
            self.instances.push(LightInstance {
                position: *position,
                color: *color,
            }.to_raw());
        }
    }
    
    pub fn read_uniform(&mut self, uniform: &Vec<LightUniform>) {
        for light in uniform.iter() {
            self.positions.push(light.pos);
            self.colors.push(light.color);
        }
    }
}