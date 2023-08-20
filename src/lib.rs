use std::time::{Duration, Instant};
use std::vec;

use camera::{Camera, CameraController, CameraUniform};
use light::LightUniform;
use model::Vertex;
use object::{DrawObject, Object, Instance, InstaceRaw};
use wgpu::ColorTargetState;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};
use cgmath::prelude::*;

mod object;
mod texture;
mod model;
mod resources;
mod camera;
mod light;

const PLAYER_ACCELERAION: f32 = 0.04;
const WINDOW_DIMENTIONS: (u32, u32) = (1280, 720);
const NUM_INSTANCES_PER_ROW: u32 = 5;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0, NUM_INSTANCES_PER_ROW as f32 * 0.5);
const SPACE_BETWEEN: u32 = 2;
const _TARGET_FPS: u32 = 60;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

struct State {
    _instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    instance_buffer1: wgpu::Buffer,
    obj_model: model::Model,
    obj_model1: model::Model,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    _light_uniform: LightUniform,
    _light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
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
                features: wgpu::Features::empty(),  
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
        let obj_model = resources::load_model("cube1.obj", &device, &queue, &object_bind_group_layout).await.unwrap();
        let obj_model1 = resources::load_model("cube2.obj", &device, &queue, &object_bind_group_layout).await.unwrap();

        // configure surface
        surface.configure(&device, &config);

        // create depth texture
        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");

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

        // lighting buffers
        let light_uniform = LightUniform::new();

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[light_uniform.to_raw()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout = LightUniform::get_bind_group_layout(&device);

        let light_bind_group = LightUniform::create_bind_group(&device, &light_bind_group_layout, &light_buffer);


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
                &light_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline( &wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState { 
                module: &shader, 
                entry_point: "vs_main", 
                buffers: &[model::ModelVertex::desc(), InstaceRaw::desc()], 
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

        // temp
        let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = cgmath::Vector3 { 
                    x: (x * SPACE_BETWEEN) as f32, 
                    y: 0.0, 
                    z: (z * SPACE_BETWEEN) as f32, 
                } - INSTANCE_DISPLACEMENT;

                let rotation = cgmath::Quaternion::one();

                Instance {
                    position,
                    rotation,
                }
            })
        }).collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        // re-create the instance buffer if new instances are added
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX
            }
        );

        let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = cgmath::Vector3 { 
                    x: (x * SPACE_BETWEEN) as f32, 
                    y: 6.0, 
                    z: (z * SPACE_BETWEEN) as f32, 
                } - INSTANCE_DISPLACEMENT;

                let rotation = cgmath::Quaternion::one();

                Instance {
                    position,
                    rotation,
                }
            })
        }).collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();

        let instance_buffer1 = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX
            }
        );

        let clear_color = wgpu::Color::BLACK;
        let dt = Duration::from_secs_f32(0.0);

        State {
            _instances: instances,
            instance_buffer,
            instance_buffer1,
            obj_model,
            obj_model1,
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            _light_uniform: light_uniform,
            _light_buffer: light_buffer,
            light_bind_group,
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

            render_pass.set_bind_group(2, &self.light_bind_group, &[]);
            render_pass.new_draw_object_instanced(&self.obj_model1, &self.instance_buffer1, &self.camera_bind_group);
            render_pass.new_draw_object_instanced(&self.obj_model, &self.instance_buffer, &self.camera_bind_group);
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