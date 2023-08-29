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
    pub color: cgmath::Vector4<f32>,
    pub pos: cgmath::Vector4<f32>,
    pub target: cgmath::Point3<f32>,
}

impl LightUniform {
    pub fn new(color: Vector3<f32>, pos: Vector3<f32>, target: cgmath::Point3<f32>) -> Self {
        Self {
            color: color.extend(1.0),
            pos: pos.extend(1.0),
            target,
        }
    }

    pub fn to_raw(&self) -> LightUniformRaw {
        LightUniformRaw {
            color: self.color.into(),
            pos: self.pos.into(),
            view_proj: {
                let light_view = cgmath::Matrix4::look_at_rh(
                    Point3::from_vec(self.pos.truncate()),
                    self.target, 
                    Vector3::unit_y(),
                );
                let light_proj = cgmath::perspective(
                    Rad(std::f32::consts::PI/2.), 
                    1.0, 
                    0.1, 
                    100.0
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
    pub view_proj: [[f32; 4]; 4],
}

use cgmath::{Rad, Vector3, Point3, EuclideanSpace};
use wgpu::SurfaceConfiguration;

use crate::model::Mesh;
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