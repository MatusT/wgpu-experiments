mod application;
mod pipelines;
mod utils;

use utils::ApplicationSkeleton;

fn main() {
    use winit::{
        event,
        event::WindowEvent,
        event_loop::{ControlFlow, EventLoop},
    };

    let event_loop = EventLoop::new();

    let mut application = application::Application::new();

    let (_window, hidpi_factor, size, surface) = {
        let window = winit::window::Window::new(&event_loop).unwrap();
        window.set_title("Hello Window");
        let hidpi_factor = window.hidpi_factor();
        let size = window.inner_size().to_physical(hidpi_factor);
        let surface = wgpu::Surface::create(&window);
        (window, hidpi_factor, size, surface)
    };

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width.round() as u32,
        height: size.height.round() as u32,
        present_mode: wgpu::PresentMode::Vsync,
    };
    let mut swap_chain = application.device().create_swap_chain(&surface, &sc_desc);

    event_loop.run(move |event, _, control_flow| match event {
        event::Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => {
            let physical = size.to_physical(hidpi_factor);

            sc_desc.width = physical.width.round() as u32;
            sc_desc.height = physical.height.round() as u32;
            swap_chain = application.device().create_swap_chain(&surface, &sc_desc);

            application.resize(sc_desc.width, sc_desc.height);
        }
        event::Event::WindowEvent { event, .. } => match event {
            WindowEvent::KeyboardInput {
                input:
                    event::KeyboardInput {
                        virtual_keycode: Some(event::VirtualKeyCode::Escape),
                        state: event::ElementState::Pressed,
                        ..
                    },
                ..
            }
            | WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {
                application.update(event);
            }
        },
        event::Event::EventsCleared => {
            let frame = swap_chain
                .get_next_texture()
                .expect("Timeout when acquiring next swap chain texture");

            application.render(&frame.view);
        }
        _ => (),
    });
}
