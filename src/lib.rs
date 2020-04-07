pub mod camera;
pub mod pipelines;

use obj::*;
use std::path::Path;
use wgpu;
use winit;

pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

pub fn load_glsl(code: &str, stage: ShaderStage) -> Vec<u32> {
    let ty = match stage {
        ShaderStage::Vertex => glsl_to_spirv::ShaderType::Vertex,
        ShaderStage::Fragment => glsl_to_spirv::ShaderType::Fragment,
        ShaderStage::Compute => glsl_to_spirv::ShaderType::Compute,
    };

    wgpu::read_spirv(glsl_to_spirv::compile(&code, ty).unwrap()).unwrap()
}

#[derive(Clone)]
pub enum ApplicationEvent {
    Resized(winit::dpi::PhysicalSize<u32>),
    Moved(winit::dpi::PhysicalPosition<u32>),
    CloseRequested,
    Destroyed,
    DroppedFile(std::path::PathBuf),
    HoveredFile(std::path::PathBuf),
    HoveredFileCancelled,
    ReceivedCharacter(char),
    Focused(bool),
    KeyboardInput {
        device_id: winit::event::DeviceId,
        input: winit::event::KeyboardInput,
        is_synthetic: bool,
    },
    CursorMoved {
        device_id: winit::event::DeviceId,
        position: winit::dpi::PhysicalPosition<f64>,
    },
    CursorEntered {
        device_id: winit::event::DeviceId,
    },
    CursorLeft {
        device_id: winit::event::DeviceId,
    },
    MouseInput {
        device_id: winit::event::DeviceId,
        state: winit::event::ElementState,
        button: winit::event::MouseButton,
    },
    TouchpadPressure {
        device_id: winit::event::DeviceId,
        pressure: f32,
        stage: i64,
    },
    AxisMotion {
        device_id: winit::event::DeviceId,
        axis: winit::event::AxisId,
        value: f64,
    },
    Added,
    Removed,
    MouseMotion {
        delta: (f64, f64),
    },
    MouseWheel {
        delta: winit::event::MouseScrollDelta,
    },
    Motion {
        axis: winit::event::AxisId,
        value: f64,
    },
    Button {
        button: winit::event::ButtonId,
        state: winit::event::ElementState,
    },
    Key(winit::event::KeyboardInput),
    ModifiersChanged(winit::event::ModifiersState),
}

impl ApplicationEvent {
    pub fn from_winit_window_event(event: &winit::event::WindowEvent) -> Self {
        match *event {
            winit::event::WindowEvent::Resized(size) => ApplicationEvent::Resized(size),
            winit::event::WindowEvent::Moved(position) => ApplicationEvent::Moved(position),
            winit::event::WindowEvent::CloseRequested => ApplicationEvent::CloseRequested,
            winit::event::WindowEvent::Destroyed => ApplicationEvent::Destroyed,
            // winit::event::WindowEvent::DroppedFile(&path) => ApplicationEvent::DroppedFile(path.clone()),
            // winit::event::WindowEvent::HoveredFile(&path) => ApplicationEvent::HoveredFile(path.clone()),
            winit::event::WindowEvent::HoveredFileCancelled => ApplicationEvent::HoveredFileCancelled,
            winit::event::WindowEvent::ReceivedCharacter(ch) => ApplicationEvent::ReceivedCharacter(ch),
            winit::event::WindowEvent::Focused(b) => ApplicationEvent::Focused(b),
            winit::event::WindowEvent::KeyboardInput {
                device_id,
                input,
                is_synthetic,
            } => ApplicationEvent::KeyboardInput {
                device_id,
                input,
                is_synthetic,
            },
            winit::event::WindowEvent::CursorMoved { device_id, position, .. } => ApplicationEvent::CursorMoved { device_id, position },
            winit::event::WindowEvent::CursorEntered { device_id } => ApplicationEvent::CursorEntered { device_id },
            winit::event::WindowEvent::CursorLeft { device_id } => ApplicationEvent::CursorLeft { device_id },
            winit::event::WindowEvent::MouseWheel { delta, .. } => ApplicationEvent::MouseWheel { delta },
            winit::event::WindowEvent::MouseInput {
                device_id, state, button, ..
            } => ApplicationEvent::MouseInput { device_id, state, button },
            winit::event::WindowEvent::TouchpadPressure {
                device_id,
                pressure,
                stage,
            } => ApplicationEvent::TouchpadPressure {
                device_id,
                pressure,
                stage,
            },
            winit::event::WindowEvent::AxisMotion { device_id, axis, value } => ApplicationEvent::AxisMotion { device_id, axis, value },
            _ => panic!("Window event not supported"),
        }
    }

    pub fn from_winit_device_event(event: &winit::event::DeviceEvent) -> Self {
        match *event {
            winit::event::DeviceEvent::Added => ApplicationEvent::Added,
            winit::event::DeviceEvent::Removed => ApplicationEvent::Removed,
            winit::event::DeviceEvent::MouseMotion { delta } => ApplicationEvent::MouseMotion { delta },
            winit::event::DeviceEvent::MouseWheel { delta } => ApplicationEvent::MouseWheel { delta },
            winit::event::DeviceEvent::Motion { axis, value } => ApplicationEvent::Motion { axis, value },
            winit::event::DeviceEvent::Button { button, state } => ApplicationEvent::Button { button, state },
            winit::event::DeviceEvent::Key(key) => ApplicationEvent::Key(key),
            winit::event::DeviceEvent::ModifiersChanged(state) => ApplicationEvent::ModifiersChanged(state),
            _ => panic!("Device event not supported"),
        }
    }
}

pub trait ApplicationSkeleton {
    fn resize(&mut self, width: u32, height: u32);

    fn update(&mut self, event: ApplicationEvent);

    fn render(&mut self, frame: &wgpu::TextureView);

    fn device(&self) -> &wgpu::Device;

    fn device_mut(&mut self) -> &mut wgpu::Device;
}

pub struct Mesh {
    pub vertices: wgpu::Buffer,
    pub vertices_len: u32,

    pub normals: wgpu::Buffer,

    pub indices: wgpu::Buffer,
    pub indices_len: u32,
}

impl Mesh {
    pub fn vertices(&self) -> &wgpu::Buffer {
        &self.vertices
    }

    pub fn vertices_len(&self) -> u32 {
        self.vertices_len
    }

    pub fn normals(&self) -> &wgpu::Buffer {
        &self.normals
    }

    pub fn indices(&self) -> &wgpu::Buffer {
        &self.indices
    }

    pub fn indices_len(&self) -> u32 {
        self.indices_len
    }

    pub fn from_obj<P: AsRef<Path>>(device: &wgpu::Device, path: P, scale: f32) -> Self {
        let file = std::io::BufReader::new(std::fs::File::open(path).unwrap());
        let obj: Obj = load_obj(file).expect("Incorrect .obj file");

        let mut vertices_cpu = Vec::new();
        for v in obj.vertices.iter() {
            vertices_cpu.push(v.position[0] * scale);
            vertices_cpu.push(v.position[1] * scale);
            vertices_cpu.push(v.position[2] * scale);
        }
        let vertices_len = vertices_cpu.len() as u32;
        let vertices = device
            .create_buffer_mapped::<f32>(vertices_cpu.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&vertices_cpu);

        let mut normals = Vec::new();
        for v in obj.vertices.iter() {
            normals.push(v.normal[0]);
            normals.push(v.normal[1]);
            normals.push(v.normal[2]);
        }
        let normals = device
            .create_buffer_mapped::<f32>(normals.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&normals);

        let indices = obj.indices;
        let indices_len = indices.len() as u32;
        let indices = device
            .create_buffer_mapped(indices.len(), wgpu::BufferUsage::INDEX)
            .fill_from_slice(&indices);

        Self {
            vertices,
            vertices_len,

            normals,

            indices,
            indices_len,
        }
    }
}
