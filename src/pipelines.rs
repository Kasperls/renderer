use std::{path::{Path, self}, rc::Rc, fs::File};

use cgmath::Angle;
use wgpu::{util::DeviceExt, DepthStencilState};

pub trait PipelineTrait {
    fn new(pipeline: wgpu::RenderPipeline, bind_groups: Vec<Rc<wgpu::BindGroup>>) -> Self 
    where Self: Sized;

    fn create(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        bind_group_containers: Vec<&crate::bindgroup::BindGroupContainer>,
        buffer_desc: Vec<wgpu::VertexBufferLayout<'static>>,
        shader_name: &str,
    ) -> Self 
    where Self: Sized,
    {

        // let shader = PipelineTrait::create_shader(device, "shaders/lowres.wgsl");
        let path = format!("C:/Users/kaspe/rust/renderer/src/{}.wgsl", shader_name);
        let data = std::fs::read_to_string(path).expect("could not open file");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(format!("{} Shader", shader_name).as_str()),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&data)).into()
        });


        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{} Pipeline layout", shader_name).as_str()),
            bind_group_layouts: bind_group_containers.iter().map(|bgc| {
                bgc.bind_group_layout.as_ref()
            }).collect::<Vec<_>>().as_slice(),
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline( &wgpu::RenderPipelineDescriptor {
            label: Some(format!("{} Render Pipeline", shader_name).as_str()),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { 
                module: &shader, 
                entry_point: "vs_main", 
                // TEMP DESC
                buffers: buffer_desc.as_slice(), 
            },
            fragment: Some(wgpu::FragmentState { 
                module: &shader, 
                entry_point: "fs_main", 
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive:  wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let bind_groups = bind_group_containers.iter().map(|bgc| {
            bgc.bind_group.clone()
        }).collect::<Vec<_>>();

        Self::new(pipeline, bind_groups)
    
    }

    // function to create a shader
    fn create_shader(&self, device: &wgpu::Device, path: &str) -> wgpu::ShaderModule {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(path),
            source: wgpu::ShaderSource::Wgsl(path.into()),
        });
        shader
    }
    
    fn get_bind_groups(&self) -> Vec<&wgpu::BindGroup>;

    fn get_pipeline(&self) -> &wgpu::RenderPipeline;

    fn create_buffers(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertices = [
            ShaderVertex { position: [-1.0, -1.0, 0.3], },
            ShaderVertex { position: [1.0, -1.0, 0.3], },
            ShaderVertex { position: [1.0, 1.0, 0.3], },
            ShaderVertex { position: [-1.0, 1.0, 0.3], },
        ];

        let indices = [
            1, 2, 0,
            2, 3, 0,
        ];

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
            label: Some("Hud Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
            label: Some("Hud Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        (vertex_buffer, index_buffer)
    }
}

pub struct LightObjectPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_groups: Vec<Rc<wgpu::BindGroup>>,
}

impl PipelineTrait for LightObjectPipeline {
    fn create(
            device: &wgpu::Device,
            config: &wgpu::SurfaceConfiguration,
            bind_group_containers: Vec<&crate::bindgroup::BindGroupContainer>,
            buffer_desc: Vec<wgpu::VertexBufferLayout<'static>>,
            shader_name: &str,
        ) -> Self 
        where Self: Sized, {

        // let shader = PipelineTrait::create_shader(device, "shaders/lowres.wgsl");
        let path = format!("C:/Users/kaspe/rust/renderer/src/{}.wgsl", shader_name);
        let data = std::fs::read_to_string(path).expect("could not open file");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(format!("{} Shader", shader_name).as_str()),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&data)).into()
        });


        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{} Pipeline layout", shader_name).as_str()),
            bind_group_layouts: bind_group_containers.iter().map(|bgc| {
                bgc.bind_group_layout.as_ref()
            }).collect::<Vec<_>>().as_slice(),
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline( &wgpu::RenderPipelineDescriptor {
            label: Some(format!("{} Render Pipeline", shader_name).as_str()),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { 
                module: &shader, 
                entry_point: "vs_main", 
                buffers: buffer_desc.as_slice(), 
            },
            fragment: Some(wgpu::FragmentState { 
                module: &shader, 
                entry_point: "fs_main", 
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive:  wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState { 
                format: crate::texture::Texture::DEPTH_FORMAT, 
                depth_write_enabled: false, 
                depth_compare: wgpu::CompareFunction::Less, 
                stencil: wgpu::StencilState::default(), 
                bias: wgpu::DepthBiasState::default(), 
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let bind_groups = bind_group_containers.iter().map(|bgc| {
            bgc.bind_group.clone()
        }).collect::<Vec<_>>();

        Self::new(pipeline, bind_groups)
            
    }

    fn new(pipeline: wgpu::RenderPipeline, bind_groups: Vec<Rc<wgpu::BindGroup>>) -> Self 
    where Self: Sized {
        Self { pipeline, bind_groups }
    }

    fn get_bind_groups(&self) -> Vec<&wgpu::BindGroup> {
        self.bind_groups.iter().map(|bg| {
            bg.as_ref()
        }).collect::<Vec<_>>()
    }

    fn get_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}

pub struct LowResPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_groups: Vec<Rc<wgpu::BindGroup>>,
}

impl PipelineTrait for LowResPipeline {
    fn new(pipeline: wgpu::RenderPipeline, bind_groups: Vec<Rc<wgpu::BindGroup>>,) -> Self 
    where Self: Sized {
        Self { pipeline, bind_groups }
    }

    fn get_bind_groups(&self) -> Vec<&wgpu::BindGroup> {
        self.bind_groups.iter().map(|bg| {
            bg.as_ref()
        }).collect::<Vec<_>>()
    }

    fn get_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShaderVertex {
    pub position: [f32; 3],
}

impl ShaderVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ShaderVertex>() as wgpu::BufferAddress,
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