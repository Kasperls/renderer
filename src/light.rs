
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LightUniform {
    pub ambient_light: cgmath::Vector4<f32>,
    pub diffuse_light: cgmath::Vector4<f32>,
    pub diffuse_light_pos: cgmath::Vector4<f32>,
    pub diffuse_light_world_pos: cgmath::Vector4<f32>,
}

impl LightUniform {
    pub fn new() -> Self {
        Self {
            ambient_light: (0.01, 0.01, 0.01, 1.0).into(),
            diffuse_light: (0.7, 0.7, 0.7, 1.0).into(),
            diffuse_light_pos: (-4.0, 4.0, 4.0, 1.0).into(),
            // original position / not updated by player movement
            diffuse_light_world_pos: (0.0, 4.0, 0.0, 1.0).into(),
        }
    }

    pub fn to_raw(&self) -> LightUniformRaw {
        LightUniformRaw {
            ambient_light: self.ambient_light.into(),
            diffuse_light: self.diffuse_light.into(),
            diffuse_light_pos: self.diffuse_light_pos.into(),
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
    pub ambient_light: [f32; 4],
    pub diffuse_light: [f32; 4],
    pub diffuse_light_pos: [f32; 4],
}