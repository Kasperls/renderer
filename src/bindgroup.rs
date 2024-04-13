use std::rc::Rc;

pub struct BindGroupContainer {
    pub bind_group: Rc<wgpu::BindGroup>,
    pub bind_group_layout: Rc<wgpu::BindGroupLayout>,
    pub buffer: Option<wgpu::Buffer>
}

impl BindGroupContainer {
    pub fn new(bind_group_layout: Rc<wgpu::BindGroupLayout>, bind_group: Rc<wgpu::BindGroup>) -> Self {
        Self {
            bind_group,
            bind_group_layout,
            buffer: None
        }
    }

    pub fn new_with_buf(bind_group_layout: Rc<wgpu::BindGroupLayout>, bind_group: Rc<wgpu::BindGroup>, buffer: wgpu::Buffer) -> Self {
        Self {
            bind_group,
            bind_group_layout,
            buffer: Some(buffer)
        }
    }
}