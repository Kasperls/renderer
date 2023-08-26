use cgmath::{Rad, Point3, InnerSpace, SquareMatrix};
use winit::event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};

use crate::{WINDOW_DIMENTIONS, PLAYER_ACCELERAION, OPENGL_TO_WGPU_MATRIX};

#[derive(Debug, Copy, Clone)]
pub enum MoveDirection {
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

pub struct CameraController {
    movement: [MoveDirection; 3],
    acc: cgmath::Vector3<f32>,
    vel: cgmath::Vector3<f32>,
    pos: cgmath::Vector3<f32>,
    yaw: cgmath::Rad<f32>,
    pitch: cgmath::Rad<f32>,
}

impl CameraController {
    pub fn new(camera: &Camera) -> Self {
        Self {
            movement: [MoveDirection::None; 3],
            acc: cgmath::vec3(0.0, 0.0, 0.0),
            vel: cgmath::vec3(0.0, 0.0, 0.0),
            pos: cgmath::vec3(camera.eye.x, camera.eye.y, camera.eye.z),
            yaw: Rad(0.0),
            pitch: Rad(0.0),
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
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
                        self.yaw += Rad(0.1);
                        true
                    }
                    VirtualKeyCode::Q => {
                        self.yaw -= Rad(0.1);
                        true
                    }
                    _ => false,
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                let delta = cgmath::vec2(position.x as f32, position.y as f32) - cgmath::vec2(WINDOW_DIMENTIONS.0 as f32 / 2.0, WINDOW_DIMENTIONS.1 as f32 / 2.0);
                self.yaw += Rad(delta.x * 0.001);
                self.pitch -= Rad(delta.y * 0.001);
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        self.acc = cgmath::vec3(
            self.movement[0].into_f32(),
            self.movement[1].into_f32(),
            self.movement[2].into_f32(),
        ) * PLAYER_ACCELERAION;

        self.acc = cgmath::Matrix3::from_angle_y(-self.yaw) * self.acc;

        self.vel += self.acc;

        self.pos += self.vel;

        camera.eye = Point3::new(self.pos.x, self.pos.y, self.pos.z);

        self.vel *= 0.7;

        self.acc = cgmath::vec3(0.0, 0.0, 0.0);

        self.pitch.0 = self.pitch.0.clamp(-cgmath::Rad(std::f32::consts::FRAC_PI_2 - 0.01).0, cgmath::Rad(std::f32::consts::FRAC_PI_2 + 0.01).0);
        
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();
        
        let forward = cgmath::Vector3::new(
            cos_pitch * cos_yaw,
            sin_pitch,
            cos_pitch * sin_yaw
        ).normalize();

        camera.target = camera.eye + forward;

        camera.up = cgmath::vec3(0.0, 1.0, 0.0);
    }
}

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }

    pub fn default(config: &wgpu::SurfaceConfiguration) -> Self {
        Camera {
            eye: (-4.0, 4.0, 2.0).into(),
            target: (0.0, -1.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    pub fn get_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor { 
                label: Some("camera_bind_group_layout"), 
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
                label: Some("camera_bind_group"), 
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
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}