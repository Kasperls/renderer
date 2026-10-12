use cgmath::{Vector2, Vector3, Vector4};
use crate::{texture, resources::load_binary};
use wgpu::util::DeviceExt;


pub struct HudContainer {
    pub hud_elements: Vec<Box<dyn HudElement>>,
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl HudContainer {
    pub fn new(
        device: &wgpu::Device, 
        queue: &wgpu::Queue, 
        config: &wgpu::SurfaceConfiguration, 
        hud_elements: Vec<Box<dyn HudElement>>, 
        // bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> Self {
        let hud_bind_group_layouts = hud_elements.iter().map(| elm | {
            elm.as_ref().get_bind_group_layout()
        }).collect::<Vec<_>>();
        // println!("{}", hud_bind_group_layouts.len());

        // create hud pipeline
        let hud_shader = device.create_shader_module( wgpu::ShaderModuleDescriptor { 
            label: Some("Hud Shader"), 
            source: wgpu::ShaderSource::Wgsl(include_str!("hud.wgsl").into()),
        });

        let hud_pipeline_layout = device.create_pipeline_layout( &wgpu::PipelineLayoutDescriptor { 
            label: Some("Hud Pipeline Layout"), 
            bind_group_layouts: hud_bind_group_layouts.as_slice(), 
            push_constant_ranges: &[], 
        });

        let pipeline = device.create_render_pipeline( &wgpu::RenderPipelineDescriptor {
            label: Some("Hud Pipeline"),
            layout: Some(&hud_pipeline_layout),
            vertex: wgpu::VertexState { 
                module: &hud_shader, 
                entry_point: "vs_main", 
                buffers: &[HudVertex::desc()], 
            },
            fragment: Some(wgpu::FragmentState { 
                module: &hud_shader, 
                entry_point: "fs_main", 
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive:  wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
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

        let (vertex_buffer, index_buffer) = Self::create_buffers(device);

        Self {
            hud_elements,
            pipeline,
            vertex_buffer,
            index_buffer,
        }

    }

    pub fn draw(&self) {
        for element in &self.hud_elements {
            element.draw();
        }
    }

    pub fn create_buffers(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertices = [
            HudVertex { position: [-1.0, -1.0, 0.3], },
            HudVertex { position: [1.0, -1.0, 0.3], },
            HudVertex { position: [1.0, 1.0, 0.3], },
            HudVertex { position: [-1.0, 1.0, 0.3], },
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

    pub fn get_vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn get_index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }
}

pub trait HudElement {
    fn draw(&self);
    fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout;
    fn get_bind_group(&self) -> &wgpu::BindGroup;
    // fn create_bind_group_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout;
    // fn create_bind_group(&self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup;
}

impl dyn HudElement {
    pub fn create_crosshair(device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) -> Box<dyn HudElement> {
        let data = load_binary("C:/Users/kaspe/rust/renderer/res/crosshair.png").expect("couldn't load image");
        let texture = texture::Texture::from_bytes(device, queue, &data, "crosshair.png").unwrap();
        let size = Vector2::new(0.1, 0.1);
        let position = Vector2::new(config.height / 2, config.width / 2);

        let bind_group_layout = device.create_bind_group_layout( &wgpu::BindGroupLayoutDescriptor { 
            label: Some("Crosshair bgl"), 
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, 
                    visibility: wgpu::ShaderStages::FRAGMENT, 
                    ty: wgpu::BindingType::Texture { 
                        multisampled: false, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    }, 
                    count: None, 
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1, 
                    visibility: wgpu::ShaderStages::FRAGMENT, 
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), 
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group( &wgpu::BindGroupDescriptor {
            label: Some("Crosshair bg"),
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
            ],
        });

        Box::new(HudElementCrosshair { position, size, texture, bind_group, bind_group_layout })
    }
}

struct HudElementCrosshair {
    pub position: Vector2<u32>,
    pub size: Vector2<f32>,
    pub texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,

}

impl HudElement for HudElementCrosshair {
    fn draw(&self) {
        todo!()
    }

    fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    // fn create_bind_group_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
    //     todo!()
    // }

    // fn create_bind_group(&self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
    //     todo!()
    // }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HudVertex {
    pub position: [f32; 3],
}

impl HudVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<HudVertex>() as wgpu::BufferAddress,
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

pub trait DrawHud<'a> {
    fn draw_hud(&mut self, hud: &'a HudContainer);
}

impl <'a, 'b> DrawHud<'b> for wgpu::RenderPass<'a> 
    where
     'b: 'a,
{
    fn draw_hud(&mut self, hud: &'a HudContainer) {
        // UPDATE THIS TO ITER OVER ALL HUD ELEMENTS AND SET THE BIND GROUPS SOMEHOW
        self.set_pipeline(&hud.pipeline);
        self.set_vertex_buffer(0, hud.vertex_buffer.slice(..));
        self.set_index_buffer(hud.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        for element in &hud.hud_elements {
            self.set_bind_group(0, element.get_bind_group(), &[]);
            self.draw_indexed(0..6, 0, 0..1);
        }
    }
}