use std::time::{Duration, Instant};
use std::vec;

use camera::{Camera, CameraController, CameraUniform};
use light::{LightUniform, LoadObjectVertexData, LightContainer};
use model::Vertex;
use object::{DrawObject, Object, InstanceRaw};
use wgpu::{ColorTargetState, DepthStencilState, BindGroupLayoutDescriptor};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};
use cgmath::{Point3, Vector3, Vector2};

mod object;
mod texture;
mod model;
mod resources;
mod camera;
mod light;

const PLAYER_ACCELERAION: f32 = 0.04;
const WINDOW_DIMENTIONS: (u32, u32) = (1280, 720);
const SHADOW_MAP_FACTOR: f32 = 10.0;
const _TARGET_FPS: u32 = 60;
const NUM_LIGHTS: u64 = 2;
const MAX_LIGHTS: u64 = 10;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

struct Globals {
    global_mx: cgmath::Matrix4<f32>,
    num_lights: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalsRaw {
    global_mx: [[f32; 4]; 4],
    num_lights: u32,
    _padding: [u32; 3],
}

impl Globals {
    fn new() -> Self {
        Self {
            global_mx: OPENGL_TO_WGPU_MATRIX,
            num_lights: NUM_LIGHTS as u32,
        }
    }

    fn to_raw(&self) -> GlobalsRaw {
        GlobalsRaw {
            global_mx: self.global_mx.into(),
            num_lights: self.num_lights,
            _padding: [0u32; 3],
        }
    }
}

struct State {
    obj_handler: object::ObjectHandler,
    obj_handler1: object::ObjectHandler,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    globals_bind_group: wgpu::BindGroup,
    lights: Vec<LightContainer>,
    uniform_light_bind_group: wgpu::BindGroup,
    _uniform_light_buffer: wgpu::Buffer,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    _config: wgpu::SurfaceConfiguration,
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

        // Create bind group layout, used to bind textures to the shader
        let object_bind_group_layout = Object::get_bind_group_layout(&device);

        // Load model
        let obj_handler = object::ObjectHandler::new(
            resources::load_model("cube1.obj", &device, &queue, &object_bind_group_layout).await.unwrap(),
            1
        );

        let obj_handler1 = object::ObjectHandler::new(
            resources::load_model("cube2.obj", &device, &queue, &object_bind_group_layout).await.unwrap(),
            2
        );

        // let obj_model = resources::load_model("cube1.obj", &device, &queue, &object_bind_group_layout).await.unwrap();
        // let obj_model1 = resources::load_model("cube2.obj", &device, &queue, &object_bind_group_layout).await.unwrap();

        // configure surface
        surface.configure(&device, &config);

        // create depth texture
        let depth_texture = texture::Texture::create_depth_texture(
            &device, 
            &config, 
            "depth_texture", 
            wgpu::CompareFunction::Less,
            Vector2::new(config.width as f32, config.height as f32),
        );

        // create camera
        let camera = Camera::default(&config);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bing_group_layout = Camera::get_bind_group_layout(&device);
        let camera_bind_group = Camera::create_bind_group(&device, &camera_bing_group_layout, &camera_buffer);

        // camera controller
        let camera_controller = CameraController::new(&camera);

        let globals = Globals::new();

        let global_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Buffer"),
            contents: bytemuck::cast_slice(&[globals.to_raw()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let globals_bind_group_layout = device.create_bind_group_layout(
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
        );

        let globals_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("globals_bind_group"),
                layout: &globals_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: global_buffer.as_entire_binding(),
                }],
            }
        );

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
                Vector3::new(0.0, 1.0, 5.0), 
                Vector3::new(-9.75137, 8.266664, -10.662715), 
                Point3::new(9.054782, 8.266664, 6.508993)
            ),
            LightUniform::new(
                Vector3::new(0.0, 1.0, 5.0), 
                Vector3::new(11.278884, 5.4666624, -28.170467), 
                Point3::new(10.712635, 5.4666624, 6.886849)
            ),
        ];

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
        


        // Create the render pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &object_bind_group_layout,
                &camera_bing_group_layout,
                &uniform_light_bind_group_layout,
                &globals_bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline( &wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
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

        let clear_color = wgpu::Color::BLACK;
        let dt = Duration::from_secs_f32(0.0);

        State {
            obj_handler,
            obj_handler1,
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            globals_bind_group,
            lights,
            uniform_light_bind_group,
            _uniform_light_buffer: uniform_light_buffer,
            surface,
            device,
            queue,
            _config: config,
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
        self.camera_controller.process_events(event)
    }

    fn update(&mut self) {
        // self.instance_buffer = self.device.create_buffer_init(
        //     &wgpu::util::BufferInitDescriptor {
        //         label: Some("Instance Buffer"),
        //         contents: bytemuck::cast_slice(&self.instances.iter().map(Instance::to_raw).collect::<Vec<_>>()),
        //         usage: wgpu::BufferUsages::VERTEX
        //     }
        // ); 

        self.window.set_title(&format!(
            "FPS: {:.2}, Camera Position: ({:.2}, {:.2}, {:.2})",
            1.0 / self.dt.as_secs_f32(),
            self.camera.eye.x,
            self.camera.eye.y,
            self.camera.eye.z
        ));

        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);

        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
        self.window.set_cursor_position(winit::dpi::PhysicalPosition::new(WINDOW_DIMENTIONS.0 / 2, WINDOW_DIMENTIONS.1 / 2)).unwrap();
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        let instance_buffer = self.obj_handler.create_instance_buffer(&self.device);
        let instance_buffer1 = self.obj_handler1.create_instance_buffer(&self.device);


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
                    self._config.height as f32 * SHADOW_MAP_FACTOR, 
                    self._config.height as f32 * SHADOW_MAP_FACTOR, 
                    0.0, 
                    1.0
                );
                
                light_render_pass.set_pipeline(&light.light_pass);
                light_render_pass.set_bind_group(0, &light.light_uniform_bind_group, &[]);
                light_render_pass.set_bind_group(1, &self.globals_bind_group, &[]);
                light_render_pass.load_object_instanced(&self.obj_handler.object, &instance_buffer);
                light_render_pass.load_object_instanced(&self.obj_handler1.object, &instance_buffer1);
            }

        }

        // self.queue.write_buffer(buffer, offset, data)
        // render final image
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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
            render_pass.set_bind_group(2, &self.uniform_light_bind_group, &[]);
            render_pass.set_bind_group(3, &self.globals_bind_group, &[]);
            render_pass.new_draw_object_instanced(
                &self.obj_handler.object, 
                &instance_buffer, 
                &self.camera_bind_group
            );
            render_pass.new_draw_object_instanced(
                &self.obj_handler1.object, 
                &instance_buffer1, 
                &self.camera_bind_group
            );
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