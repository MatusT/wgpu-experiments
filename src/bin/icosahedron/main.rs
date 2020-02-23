mod application;

use wgpu_experiments::{ApplicationEvent, ApplicationSkeleton};

fn main() {
    use winit::{
        event,
        event::DeviceEvent,
        event::WindowEvent,
        event_loop::{ControlFlow, EventLoop},
    };

    let event_loop = EventLoop::new();

    let (window, size, surface) = {
        let window = winit::window::Window::new(&event_loop).unwrap();
        window.set_title("Hello world");
        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);
        (window, size, surface)
    };

    let mut application = application::Application::new(size.width, size.height);

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Vsync,
    };
    let mut swap_chain = application.device().create_swap_chain(&surface, &sc_desc);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            event::Event::MainEventsCleared => window.request_redraw(),
            event::Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                swap_chain = application.device().create_swap_chain(&surface, &sc_desc);

                application.resize(sc_desc.width, sc_desc.height);
            }
            event::Event::WindowEvent { event, .. } => {
                match event {
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
                    _ => {}
                };
                application.update(ApplicationEvent::WindowEvent(event));
            }
            event::Event::RedrawRequested(_) => {
                let frame = swap_chain
                    .get_next_texture()
                    .expect("Timeout when acquiring next swap chain texture");

                application.render(&frame.view);
            }
            event::Event::DeviceEvent { event, .. } => {
                application.update(ApplicationEvent::DeviceEvent(event));
            }
            _ => {}
        }
    });
}
