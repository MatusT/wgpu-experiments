use crate::ApplicationEvent;
use nalgebra_glm as glm;
use safe_transmute::TriviallyTransmutable;
use winit;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CameraUbo {
    pub projection: glm::Mat4,
    pub view: glm::Mat4,
    pub projection_view: glm::Mat4,
}

unsafe impl TriviallyTransmutable for CameraUbo {}
pub trait Camera {
    fn resize(&mut self, aspect: f32, fov: f32, near: f32);
    fn update(&mut self, event: ApplicationEvent);
    fn ubo(&self) -> CameraUbo;
    fn set_speed(&mut self, speed: f32);
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
        let projection = glm::reversed_infinite_perspective_rh_zo(aspect, fov, near);
        let view = glm::look_at(&glm::vec3(0.0, 0.0, 0.0), &glm::vec3(0.0, 0.0, 0.0), &glm::vec3(0.0, 1.0, 0.0));
        let projection_view = projection * view;

        Self {
            ubo: CameraUbo {
                projection,
                view,
                projection_view,
            },

            yaw: -90.0,
            pitch: 0.0,
            distance: 1.0,

            speed: 1.0,
            mouse_pressed: false,
        }
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
                    self.distance += change * self.speed;
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

        let eye = self.distance * self.direction_vector();
        self.ubo.view = glm::look_at(&eye, &glm::vec3(0.0, 0.0, 0.0), &glm::vec3(0.0, 1.0, 0.0));
        self.ubo.projection_view = self.ubo.projection * self.ubo.view;
    }

    fn ubo(&self) -> CameraUbo {
        self.ubo
    }

    fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }
}
