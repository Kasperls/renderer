use crate::{OPENGL_TO_WGPU_MATRIX, NUM_LIGHTS};

pub struct Globals {
    pub global_mx: cgmath::Matrix4<f32>,
    pub num_lights: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalsRaw {
    global_mx: [[f32; 4]; 4],
    num_lights: u32,
    _padding: [u32; 3],
}

impl Globals {
    pub fn new() -> Self {
        Self {
            global_mx: OPENGL_TO_WGPU_MATRIX,
            num_lights: NUM_LIGHTS as u32,
        }
    }

    pub fn to_raw(&self) -> GlobalsRaw {
        GlobalsRaw {
            global_mx: self.global_mx.into(),
            num_lights: self.num_lights,
            _padding: [0u32; 3],
        }
    }

    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("globals_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            }
        )
    }

    pub fn create_bind_group(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, buffer: wgpu::Buffer) -> wgpu::BindGroup {
        device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("globals_bind_group"),
                layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            }
        )
    }
}