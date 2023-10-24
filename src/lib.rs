use std::pin::Pin;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::{vec, fs};

use csv::DeserializeError;
use serde::{Deserialize, Deserializer, de};
use serde::de::Visitor;

use camera::{Camera, CameraController, CameraUniform};
use hud::{DrawHud, HudContainer};
use light::{LightUniform, LoadObjectVertexData, LightContainer, DrawLightVertexData};
use model::Vertex;
use object::{DrawObject, Object, InstanceRaw, ObjectHandler};
use pipelines::{PipelineTrait, LightObjectPipeline};
use wgpu::{ColorTargetState, DepthStencilState, BindGroupLayoutDescriptor};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};
use cgmath::{Point3, Vector3, Vector2, Angle, Array};

mod object;
mod texture;
mod model;
mod resources;
mod camera;
mod light;
mod globals;
mod hud;
mod room;
mod hitbox;
mod bindgroup;
mod pipelines;

const PLAYER_ACCELERAION: f32 = 0.04;
const WINDOW_DIMENTIONS: (u32, u32) = (1280, 720);
const SHADOW_MAP_FACTOR: f32 = 5.0;
const _TARGET_FPS: u32 = 10;
const NUM_LIGHTS: u64 = 2;
const MAX_LIGHTS: u64 = 10;

const PIXEL_RES: u32 = 1;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

struct State {
    pixel_res: u32,
    hud_container: hud::HudContainer,
    object_handler: object::ObjectHandler,
    light_handler: light::LightHandler,
    camera_container: camera::CameraContainer,
    globals_bind_group: wgpu::BindGroup,
    lights: Vec<LightContainer>,
    uniform_light_bind_group_container: bindgroup::BindGroupContainer,
    lowres_texture: wgpu::Texture,
    low_res_pipeline: pipelines::LowResPipeline,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    clear_color: wgpu::Color,
    dt: Duration,
}

impl State {
    async fn new(window: Window) -> Self {

        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::MULTIVIEW,  
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults() 
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
        .copied().find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);
        
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        // draw to this before drawing to the window surface later
        let lowres_texture = device.create_texture(&wgpu::TextureDescriptor { 
            label: Some("Pre renderer texture"), 
            size: wgpu::Extent3d {
                width: size.width / PIXEL_RES,
                height: size.height / PIXEL_RES,
                depth_or_array_layers: 1,
            }, 
            mip_level_count: 1, 
            sample_count: 1, 
            dimension: wgpu::TextureDimension::D2, 
            format: wgpu::TextureFormat::Bgra8UnormSrgb, 
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, 
            view_formats: &[],
        });

        // Create bind group layout, used to bind textures to the shader
        let object_bind_group_layout = Object::get_textured_bind_group_layout(&device);

        // Load model
        let object_handler = object::ObjectHandler::new(
            resources::load_object("cube_smart.obj", &device).await.unwrap(),
            ObjectHandler::load_texture_atlas("cube_smart.png", &device, &queue).unwrap(),
        );

        // configure surface
        surface.configure(&device, &config);

        // create depth texture
        let depth_texture = texture::Texture::create_depth_texture(
            &device, 
            &config, 
            "depth_texture", 
            wgpu::CompareFunction::Less,
            Vector2::new((config.width / PIXEL_RES) as f32, (config.height / PIXEL_RES) as f32),
        );

        // create camera
        let camera_container = camera::CameraContainer::new(&device, &config);

        let globals = globals::Globals::new();

        let global_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Buffer"),
            contents: bytemuck::cast_slice(&[globals.to_raw()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let globals_bind_group_layout = globals::Globals::create_bind_group_layout(&device);

        let globals_bind_group = globals::Globals::create_bind_group(&device, &globals_bind_group_layout, global_buffer);
        
        // Move into light.rs
        // light bind group layout used in light pass
        let light_uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("light_uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }]
        });

        // each light instance creates a view into the depth texture
        let view_size = wgpu::Extent3d {
            width: (config.height as f32 * SHADOW_MAP_FACTOR) as u32,
            height: (config.height as f32 * SHADOW_MAP_FACTOR) as u32,
            depth_or_array_layers: MAX_LIGHTS as u32,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("light_depth_texture"),
            size: view_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let light_depth_texture = device.create_texture(&desc);

        // lighting buffers
        let light_uniforms = vec![
            LightUniform::new(
                Vector3::new(0.9, 0.2, 0.7), 
                Point3::new(-2.7181466, 13.20014, -3.9281557), 
Vector3::new(0.53910136, -0.6668692, 0.51444644)
            ),
            LightUniform::new(
                Vector3::new(0.2, 1.0, 0.3), 
                Point3::new(19.697975, 13.200004, 6.5290422), 
Vector3::new(-0.66862667, -0.7110567, 0.21756993)
            ),
        ];

        // let light_object_bind_group_layout = 

        let mut light_handler = light::LightHandler::new(
            resources::load_light_obj("C:/Users/kasp9/rust/renderer/res/ball.obj", &device).await.unwrap(),
            LightObjectPipeline::create(
                &device, 
                &config,
                vec![camera_container.get_bind_group_container()], 
                vec![light::LightVertex::desc(), light::LightInstanceRaw::desc()], 
                "light-obj"
            ),
        );

        light_handler.read_uniform(&light_uniforms);
        light_handler.create_instances();

        let mut lights: Vec<LightContainer> = vec![];
        // refactor into .map()
        for (i, light) in light_uniforms.iter().enumerate() {
            let light_buffer: wgpu::Buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Light Buffer"),
                contents: bytemuck::cast_slice(&[light.to_raw()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let light_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("light_uniform_bind_group"),
                layout: &light_uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_buffer.as_entire_binding(),
                }],
            });

            let light_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { 
                label: Some("light_shader"), 
                source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
            });
        
            let light_pass_layout = device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor { 
                    label: Some("light_pass_layout"), 
                    bind_group_layouts: &[
                        &light_uniform_bind_group_layout,
                        &globals_bind_group_layout
                    ],
                    push_constant_ranges: &[],
                }
            );
        
            let light_pass = device.create_render_pipeline( &wgpu::RenderPipelineDescriptor {
                label: Some("light_pass"),
                layout: Some(&light_pass_layout),
                vertex: wgpu::VertexState {
                    module: &light_shader,
                    entry_point: "vs_main",
                    buffers: &[
                        model::ModelVertex::desc(), InstanceRaw::desc()
                    ],
                },
                fragment: None,
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: device
                        .features()
                        .contains(wgpu::Features::DEPTH_CLIP_CONTROL),
                    ..Default::default()
                },
                depth_stencil: Some(DepthStencilState {
                    format: texture::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: Default::default(),
                    bias: wgpu::DepthBiasState {
                        constant: 2,
                        slope_scale: 2.0,
                        clamp: 0.0,
                    },
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

            let light_depth_texture_view = light_depth_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("light_depth_texture_view"),
                format: None,
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: i as u32,
                array_layer_count: Some(1) //Some(NUM_LIGHTS as u32),
            });
            
            let light_container = LightContainer {
                index: i,
                light: *light,
                light_buffer,
                light_uniform_bind_group,
                light_pass,
                light_depth_texture_view,
            };

            lights.push(light_container);
        };

        // compare function is the one used in the shader
        let light_depth_texture_sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                label: Some("light_depth_texture_sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual),
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );

        let uniform_light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(
                &light_uniforms
                .iter()
                .map(
                    |l| l.to_raw()
                ).collect::<Vec<_>>()
            ),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        });

        // Move into light.rs?
        // light bind group layout used in final render pass
        let uniform_light_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("uniform_light_and_depth_texture_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1, 
                visibility: wgpu::ShaderStages::FRAGMENT, 
                ty: wgpu::BindingType::Texture { 
                    multisampled: false, 
                    view_dimension: wgpu::TextureViewDimension::D2Array, 
                    sample_type: wgpu::TextureSampleType::Depth,
                }, 
                count: None, 
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT, 
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison), 
                count: None,
            },
            ],
        });

        let uniform_light_depth_texture_view = light_depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let uniform_light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform_light_and_depth_texture_bind_group"),
            layout: &uniform_light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_light_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&uniform_light_depth_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&light_depth_texture_sampler),
            },
            ],
        });

        let uniform_light_bind_group_container = bindgroup::BindGroupContainer::new_with_buf(
            Rc::new(light_uniform_bind_group_layout),
            Rc::new(uniform_light_bind_group),
            uniform_light_buffer,
        );
        


        // Create the render pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("frw Render Pipeline Layout"),
            bind_group_layouts: &[
                &object_bind_group_layout,
                &camera_container.bind_group_container.bind_group_layout,
                &uniform_light_bind_group_layout,
                &globals_bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline( &wgpu::RenderPipelineDescriptor {
            label: Some("frw Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState { 
                module: &shader, 
                entry_point: "vs_main", 
                buffers: &[model::ModelVertex::desc(), InstanceRaw::desc()], 
            },
            fragment: Some(wgpu::FragmentState { 
                module: &shader, 
                entry_point: "fs_main", 
                targets: &[Some(ColorTargetState {
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
                format: texture::Texture::DEPTH_FORMAT, 
                depth_write_enabled: true, 
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

        /* 
        
        */
        let lowres_sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                label: None,
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                compare: None,
                anisotropy_clamp: 1,
                border_color: None,
            }
        );

        
        let lowres_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("lowres_bind_group_layout"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: None, 
                    },
                    count: None,
                }
                ],
            });
            
            let lowres_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Lowres Buffer"),
                contents: bytemuck::cast_slice(&[size.width as f32, size.height as f32]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            
            let lowres_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("lowres_bind_group"),
                layout: &lowres_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&lowres_texture.create_view(&wgpu::TextureViewDescriptor::default())),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&lowres_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: lowres_buffer.as_entire_binding(),
                    }
                    ],
                });
                
                
        let low_res_bindgroup_container = bindgroup::BindGroupContainer::new(
            Rc::new(lowres_bind_group_layout),
            Rc::new(lowres_bind_group),
        );

        let low_res_pipeline = pipelines::LowResPipeline::create(
            &device, 
            &config, 
            vec![&low_res_bindgroup_container],
            vec![pipelines::ShaderVertex::desc()],
            "lowres",
        );

        // hud elements

        let hud_elements = vec![
            <dyn hud::HudElement>::create_crosshair(&device, &queue, &config)
        ];

        let hud_container = hud::HudContainer::new(
            &device, 
            &queue, 
            &config, 
            hud_elements,
        );

        let clear_color = wgpu::Color::TRANSPARENT;
        let dt = Duration::from_secs_f32(0.0);

        State {
            pixel_res: PIXEL_RES,
            hud_container,
            object_handler,
            light_handler,
            camera_container,
            globals_bind_group,
            lights,
            uniform_light_bind_group_container,
            surface,
            lowres_texture,
            low_res_pipeline,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            depth_texture,
            clear_color,
            dt
        }
    
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, _new_size: winit::dpi::PhysicalSize<u32>) {
        // if new_size.width > 0 && new_size.height > 0 {
        //     self.size = new_size;
        //     self.config.width = new_size.width;
        //     self.config.height = new_size.height;
        //     self.surface.configure(&self.device, &self.config);
        //     self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        // }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        
        match event {
            WindowEvent::MouseWheel { 
                delta,
                ..
            } => {
                match delta {
                    MouseScrollDelta::LineDelta(_, y) => {
                        self.camera_container.controller.speed += *y as f32 / 8.0;
                        let tmp = self.pixel_res as i32 + *y as i32;
                        self.pixel_res = tmp as u32;
                        if self.pixel_res == 0 {
                            self.pixel_res = 1;
                        }

                    },
                    _ => {},
                }
            },
            _ => {}
        }

        self.camera_container.controller.process_events(event)
    }

    fn update(&mut self) {
        // let x = self.lights[0].light.pos.x;
        // let y = self.lights[0].light.pos.z;
        
        // self.lights[0].light.pos.x = x * f32::cos(0.01) - y * f32::sin(0.01);
        // self.lights[0].light.pos.z = x * f32::sin(0.01) + y * f32::cos(0.01);

        // self.queue.write_buffer(&self.uniform_light_buffer, 0, bytemuck::cast_slice(&[self.lights[0].light.to_raw()]));

        self.window.set_title(&format!(
            "FPS: {:.2}, Camera Position: ({:.2?})",
            1.0 / self.dt.as_secs_f32(),
            self.camera_container.get_coords()
        ));

        self.camera_container.update(&self.object_handler.positions);
        self.camera_container.write_to_buffer(&mut self.queue);

        self.window.set_cursor_position(winit::dpi::PhysicalPosition::new(WINDOW_DIMENTIONS.0 / 2, WINDOW_DIMENTIONS.1 / 2)).unwrap();
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        // the final view
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // the scene is rendered to this
        let pre_view = self.lowres_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        let obj_instance_buffer = self.object_handler.create_instance_buffer(&self.device);

        // calculate zdepth buffer
        for light in &mut self.lights {
            {   
                let mut light_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Light Render Pass"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &light.light_depth_texture_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0), 
                            store: true,
                        }),
                        stencil_ops: None,
                    })
                });
    
                light_render_pass.set_viewport(
                    0.0, 
                    0.0, 
                    self.config.height as f32 * SHADOW_MAP_FACTOR, 
                    self.config.height as f32 * SHADOW_MAP_FACTOR, 
                    0.0, 
                    1.0
                );
                
                light_render_pass.set_pipeline(&light.light_pass);
                light_render_pass.set_bind_group(0, &light.light_uniform_bind_group, &[]);
                light_render_pass.set_bind_group(1, &self.globals_bind_group, &[]);
                light_render_pass.load_object_instanced(&self.object_handler.object, &obj_instance_buffer);
            }
        }

        // self.queue.write_buffer(buffer, offset, data)
        // render final image
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: {
                        match PIXEL_RES {
                            1 => &view,
                            _ => &pre_view,
                        }
                    },
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0), 
                        store: true,
                    }),
                    stencil_ops: None,
                })
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(2, &self.uniform_light_bind_group_container.bind_group, &[]);
            render_pass.set_bind_group(3, &self.globals_bind_group, &[]);
            render_pass.draw_object_instanced(
                &self.object_handler,
                &obj_instance_buffer,
                &self.camera_container.get_bind_group(),
            );
        }

        // render lights
        {
            let mut light_obj_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Light Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: {
                        match PIXEL_RES {
                            1 => &view,
                            _ => &pre_view,
                        }
                    },
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0), 
                        store: true,
                    }),
                    stencil_ops: None,
                })
            });

            light_obj_pass.set_pipeline(self.light_handler.pipeline.get_pipeline());
            light_obj_pass.set_bind_group(0, self.camera_container.get_bind_group(), &[]);
            light_obj_pass.draw_light_object_instanced(&self.light_handler.object, &self.light_handler.create_instance_buffer(&self.device))            
        }

        // pixel render pass
        match PIXEL_RES {
            1 => {},
            _ => {
                let (vertex_buf, index_buf) = HudContainer::create_buffers(&self.device);
                {
                    let mut lowres_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Pixel Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(self.clear_color),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                    });
                    
                    lowres_render_pass.set_pipeline(&self.low_res_pipeline.get_pipeline());
                    lowres_render_pass.set_bind_group(0, &self.low_res_pipeline.get_bind_groups()[0], &[]);
                    lowres_render_pass.set_vertex_buffer(0, vertex_buf.slice(..));
                    lowres_render_pass.set_index_buffer(index_buf.slice(..), wgpu::IndexFormat::Uint32);
                    lowres_render_pass.draw_indexed(0..6, 0, 0..1);
                }
            },
        }

        // hud render pass
        {
            let mut hud_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Hud Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, 
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            
            // hud_render_pass.draw_hud(&self.hud_container);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())

    }
}

pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
    .with_inner_size(PhysicalSize::new(WINDOW_DIMENTIONS.0, WINDOW_DIMENTIONS.1))
    .build(&event_loop)
    .unwrap();

    window.set_cursor_visible(false);

    let mut state = State::new(window).await;
    let mut last_recorded_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    },
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                let now = Instant::now();
                state.dt = now - last_recorded_time;
                last_recorded_time = now;

                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }

            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                state.window().request_redraw();
            }
            _ => {}

            
        }
    });
}