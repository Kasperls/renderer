use std::vec;

use model::Vertex;
use wgpu::{ColorTargetState, PrimitiveState};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window, self},
};
use image::GenericImageView;
use cgmath::{SquareMatrix, Point3};
use cgmath::prelude::*;

mod texture;
mod model;
mod resources;
use model::DrawModel;


const PLAYER_ACCELERAION: f32 = 0.01;
const WINDOW_DIMENTIONS: (u32, u32) = (1280, 720);
const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0, NUM_INSTANCES_PER_ROW as f32 * 0.5);
const SPACE_BETWEEN: u32 = 3;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    fn to_raw(&self) -> InstaceRaw {
        InstaceRaw {
            model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)).into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstaceRaw {
    model: [[f32; 4]; 4],
}

impl InstaceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstaceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum MoveDirection {
    Positive,
    Negative,
    None,
}

impl MoveDirection {
    fn into_f32(&self) -> f32 {
        match self {
            MoveDirection::Positive => 1.0,
            MoveDirection::Negative => -1.0,
            MoveDirection::None => 0.0,
        }
    }

    fn from_bool(bool: bool, dir: i32) -> Self {
        match bool {
            true => {
                if dir < 0 {
                    MoveDirection::Negative
                } else {
                    MoveDirection::Positive
                }
            },
            false => MoveDirection::None,
        }
    }
}

struct CameraController {
    movement: [MoveDirection; 3],
    acc: cgmath::Vector3<f32>,
    vel: cgmath::Vector3<f32>,
    pos: cgmath::Vector3<f32>,
    yaw: f32,
    pitch: f32,
}

impl CameraController {
    fn new(camera: &Camera) -> Self {
        Self {
            movement: [MoveDirection::None; 3],
            acc: cgmath::vec3(0.0, 0.0, 0.0),
            vel: cgmath::vec3(0.0, 0.0, 0.0),
            pos: cgmath::vec3(camera.eye.x, camera.eye.y, camera.eye.z),
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.movement[0] = MoveDirection::from_bool(is_pressed, 1);
                        true
                    }
                    VirtualKeyCode::Space => {
                        self.movement[1] = MoveDirection::from_bool(is_pressed, 1);
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Left => {
                        self.movement[2] = MoveDirection::from_bool(is_pressed, 1);
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.movement[0] = MoveDirection::from_bool(is_pressed, -1);
                        true
                    }
                    VirtualKeyCode::LShift => {
                        self.movement[1] = MoveDirection::from_bool(is_pressed, -1);
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Right => {
                        self.movement[2] = MoveDirection::from_bool(is_pressed, -1);
                        true
                    }
                    VirtualKeyCode::E => {
                        self.yaw += 0.1;
                        true
                    }
                    VirtualKeyCode::Q => {
                        self.yaw -= 0.1;
                        true
                    }
                    _ => false,
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                let delta = cgmath::vec2(position.x as f32, position.y as f32) - cgmath::vec2(WINDOW_DIMENTIONS.0 as f32 / 2.0, WINDOW_DIMENTIONS.1 as f32 / 2.0);
                self.yaw += delta.x * 0.001;
                self.pitch -= delta.y * 0.001;
                true
            }
            _ => false,
        }
    }

    fn update_camera(&mut self, camera: &mut Camera) {
        // VIRKER IKKE
        self.acc = cgmath::vec3(
            self.movement[0].into_f32(),
            self.movement[1].into_f32(),
            self.movement[2].into_f32(),
        ) * PLAYER_ACCELERAION;

        self.acc = cgmath::Matrix3::from_angle_y(cgmath::Rad(-self.yaw)) * self.acc;

        self.vel += self.acc;

        self.pos += self.vel;

        camera.eye = Point3::new(self.pos.x, self.pos.y, self.pos.z);

        self.vel *= 0.7;

        self.acc = cgmath::vec3(0.0, 0.0, 0.0);

        camera.target = Point3::new(
            self.pos.x + self.yaw.cos(),
            self.pos.y + self.pitch.sin(),
            self.pos.z + self.yaw.sin(),
        );

        camera.up = cgmath::vec3(0.0, 1.0, 0.0);
    }
}

struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

struct State {
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    obj_model: model::Model,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
    diffuse_bind_group_vec: Vec<wgpu::BindGroup>,
    diffuse_texture_vec: Vec<texture::Texture>,
    depth_texture: texture::Texture,
    is_tab_pressed: bool,
    clear_color: wgpu::Color,
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
        let texture_bind_group_layout = 
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0, 
                        visibility: wgpu::ShaderStages::FRAGMENT, 
                        ty: wgpu::BindingType::Texture { 
                            multisampled: false, 
                            view_dimension: wgpu::TextureViewDimension::D2, 
                            sample_type: wgpu::TextureSampleType::Float { 
                                filterable: true 
                            }, 
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
                label: Some("texture_bind_group_layout"), 
            }
        );

        let obj_model = resources::load_model("cube.obj", &device, &queue, &texture_bind_group_layout).await.unwrap();


        // Load texture
        let mut diffuse_texture_vec: Vec<texture::Texture> = vec![];

        surface.configure(&device, &config);
        let diffuse_bytes = include_bytes!("assets/happy-tree.png");
        let diffuse_texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();
        
        diffuse_texture_vec.push(diffuse_texture);
        
        let diffuse_bytes = include_bytes!("assets/stone.png");
        let diffuse_texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "stone.png").unwrap();
        
        diffuse_texture_vec.push(diffuse_texture);

        // create depth texture
        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        // Create bind group

        let mut diffuse_bind_group_vec: Vec<wgpu::BindGroup> = vec![];

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            layout: &texture_bind_group_layout, 
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0, 
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_vec[0].view), 
                },
                wgpu::BindGroupEntry {
                    binding: 1, 
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture_vec[0].sampler), 
                },
            ],
            label: Some("diffuse_bind_group"), 
        });

        diffuse_bind_group_vec.push(diffuse_bind_group);

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            layout: &texture_bind_group_layout, 
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0, 
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_vec[1].view), 
                },
                wgpu::BindGroupEntry {
                    binding: 1, 
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture_vec[1].sampler), 
                },
            ],
            label: Some("diffuse_bind_group"), 
        });

        diffuse_bind_group_vec.push(diffuse_bind_group);

        // create camera and camera uniform

        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bing_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor { 
                label: Some("camera_bind_group_layout"), 
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ] 
            }
        );

        let camera_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor { 
                label: Some("camera_bind_group"), 
                layout: &camera_bing_group_layout, 
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }
                ] 
            }
        );

        // camera controller

        let camera_controller = CameraController::new(&camera);

        // Create the render pipeline

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &camera_bing_group_layout,
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

        let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = cgmath::Vector3 { 
                    x: (x * SPACE_BETWEEN) as f32, 
                    y: 0.0, 
                    z: (z * SPACE_BETWEEN) as f32, 
                } - INSTANCE_DISPLACEMENT;

                let rotation = if position.is_zero() {
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.clone().normalize(), cgmath::Deg(45.0))
                };
                
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
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let clear_color = wgpu::Color::BLACK;

        State {
            instances,
            instance_buffer,
            obj_model,
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            diffuse_bind_group_vec,
            diffuse_texture_vec,
            depth_texture,
            clear_color,
            is_tab_pressed: false,
        }
    
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
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

            let index = if self.is_tab_pressed { 0 } else { 0 };

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_pipeline(&self.render_pipeline);

            // render_pass.set_bind_group(0, &self.diffuse_bind_group_vec[index], &[]);
            // render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            render_pass.draw_model_instanced(&self.obj_model, 0..self.instances.len() as u32, &self.camera_bind_group);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())


    }
}


// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
// struct Vertex {
//     position: [f32; 3],
//     tex_coords: [f32; 2],
// }

// impl Vertex {
//     fn desc() -> wgpu::VertexBufferLayout<'static> {
//         wgpu::VertexBufferLayout {
//             array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, // 1.
//             step_mode: wgpu::VertexStepMode::Vertex, // 2.
//             attributes: &[ // 3.
//                 wgpu::VertexAttribute {
//                     offset: 0, // 4.
//                     shader_location: 0, // 5.
//                     format: wgpu::VertexFormat::Float32x3, // 6.
//                 },
//                 wgpu::VertexAttribute {
//                     offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
//                     shader_location: 1,
//                     format: wgpu::VertexFormat::Float32x2,
//                 }
//             ]
//         }
//     }
// }

// const VERTICES: &[Vertex] = &[
//     Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], }, // A
//     Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], }, // B
//     Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397], }, // C
//     Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914], }, // D
//     Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], }, // E
// ];

// const INDICES: &[u16] = &[
//     0, 1, 4,
//     1, 2, 4,
//     2, 3, 4,
// ];
 
 

pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
    .with_inner_size(PhysicalSize::new(WINDOW_DIMENTIONS.0, WINDOW_DIMENTIONS.1))
    .build(&event_loop)
    .unwrap();

    window.set_cursor_visible(false);

    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| { 
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
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
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}