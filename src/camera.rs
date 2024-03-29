use crate::ApplicationEvent;
use bytemuck::*;
use nalgebra_glm as glm;
use winit;
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CameraUbo {
    pub projection: glm::Mat4,
    pub view: glm::Mat4,
    pub projection_view: glm::Mat4,
    pub position: glm::Vec4,
}

unsafe impl Zeroable for CameraUbo {}
unsafe impl Pod for CameraUbo {}
pub trait Camera {
    fn resize(&mut self, aspect: f32, fov: f32, near: f32);
    fn update(&mut self, event: ApplicationEvent);
    fn ubo(&mut self) -> CameraUbo;
    fn set_speed(&mut self, speed: f32);
    fn set_distance(&mut self, distance: f32);
}

pub struct RotationCamera {
    ubo: CameraUbo,

    yaw: f32,
    pitch: f32,
    distance: f32,

    speed: f32,
    mouse_pressed: bool,
}

impl RotationCamera {
    pub fn new(aspect: f32, fov: f32, near: f32) -> RotationCamera {
        let distance = 1500.0;
        let projection = glm::reversed_infinite_perspective_rh_zo(aspect, fov, near);

        let mut camera = RotationCamera {
            ubo: CameraUbo {
                projection,
                view: glm::one(),
                projection_view: glm::one(),
                position: glm::zero(),
            },

            yaw: -90.0,
            pitch: 0.0,
            distance,

            speed: 100.0,
            mouse_pressed: false,
        };

        let eye = camera.distance * camera.direction_vector();
        camera.ubo.view = glm::look_at(&eye, &glm::vec3(0.0, 0.0, 0.0), &glm::vec3(0.0, 1.0, 0.0));
        camera.ubo.projection_view = camera.ubo.projection * camera.ubo.view;
        camera.ubo.position = glm::vec4(eye[0], eye[1], eye[2], 0.0);

        camera
    }

    fn direction_vector(&self) -> glm::Vec3 {
        let yaw = self.yaw.to_radians();
        let pitch = self.pitch.to_radians();

        glm::vec3(yaw.cos() * pitch.cos(), pitch.sin(), yaw.sin() * pitch.cos())
    }
}

impl Camera for RotationCamera {
    fn resize(&mut self, aspect: f32, fov: f32, near: f32) {
        self.ubo.projection = glm::perspective(aspect, fov, near, 100.0);
    }

    fn update<'a>(&mut self, event: ApplicationEvent) {
        match event {
            ApplicationEvent::MouseWheel { delta, .. } => {
                if let winit::event::MouseScrollDelta::LineDelta(_, change) = delta {
                    self.distance -= change * self.speed;
                }
            }
            ApplicationEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left {
                    if state == winit::event::ElementState::Pressed {
                        self.mouse_pressed = true;
                    } else {
                        self.mouse_pressed = false;
                    }
                }
            }
            ApplicationEvent::MouseMotion { delta: (x, y) } => {
                if self.mouse_pressed {
                    self.yaw += x as f32;
                    self.pitch += y as f32;
                }
            }
            _ => {}
        };
    }

    fn ubo(&mut self) -> CameraUbo {
        let eye = self.distance * self.direction_vector();
        self.ubo.view = glm::look_at(&eye, &glm::vec3(0.0, 0.0, 0.0), &glm::vec3(0.0, 1.0, 0.0));
        self.ubo.projection_view = self.ubo.projection * self.ubo.view;

        self.ubo
    }

    fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    fn set_distance(&mut self, distance: f32) {
        self.distance = distance;
    }
}
