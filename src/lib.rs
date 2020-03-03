pub mod camera;
pub mod pipelines;

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

// pub enum ApplicationEvent<'a> {
//     DeviceEvent(winit::event::DeviceEvent),
//     WindowEvent(winit::event::WindowEvent<'a>),
// }

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
        modifiers: winit::event::ModifiersState,
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
        modifiers: winit::event::ModifiersState,
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
            winit::event::WindowEvent::CursorMoved {
                device_id,
                position,
                modifiers,
            } => ApplicationEvent::CursorMoved {
                device_id,
                position,
                modifiers,
            },
            winit::event::WindowEvent::CursorEntered { device_id } => ApplicationEvent::CursorEntered { device_id },
            winit::event::WindowEvent::CursorLeft { device_id } => ApplicationEvent::CursorLeft { device_id },
            winit::event::WindowEvent::MouseWheel {
                delta,
                ..
            } => ApplicationEvent::MouseWheel {
                delta,
            },
            winit::event::WindowEvent::MouseInput {
                device_id,
                state,
                button,
                modifiers,
            } => ApplicationEvent::MouseInput {
                device_id,
                state,
                button,
                modifiers,
            },
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
