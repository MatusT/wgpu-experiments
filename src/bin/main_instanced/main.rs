mod application;
mod grid;

use wgpu_experiments::{ApplicationEvent, ApplicationSkeleton};

fn main() {
    use winit::{
        event,
        event::WindowEvent,
        event_loop::{ControlFlow, EventLoop},
    };

    let instance = wgpu::Instance::new();

    // Initialize winit
    let event_loop = EventLoop::new();

    let (window, size, surface) = {
        let window = winit::window::Window::new(&event_loop).unwrap();
        window.set_inner_size(winit::dpi::LogicalSize { width: 1920, height: 1080 });
        window.set_title("AABB Finding");
        let size = window.inner_size();
        let surface = unsafe { instance.create_surface(&window) };
        (window, size, surface)
    };

    // Initialize the graphics scene
    let mut application = futures::executor::block_on(application::Application::new(size.width, size.height, &instance, &surface));

    // Initialize swapchain
    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };
    let mut swap_chain = application.device().create_swap_chain(&surface, &sc_desc);

    let mut ui_on = false;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            // Handle resize event as a special case
            event::Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                sc_desc.width = size.width;
                sc_desc.height = size.height;

                swap_chain = application.device().create_swap_chain(&surface, &sc_desc);

                application.resize(sc_desc.width, sc_desc.height);
            }
            // Gather window + device events
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
                    WindowEvent::KeyboardInput {
                        input:
                            event::KeyboardInput {
                                virtual_keycode: Some(event::VirtualKeyCode::U),
                                state: event::ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        ui_on = !ui_on;
                    }
                    _ => {}
                };

                let event = event.to_static().unwrap();

                // Send window event to the graphics scene
                application.update(ApplicationEvent::from_winit_window_event(&event));
            }
            event::Event::DeviceEvent { event, .. } => {
                match event {
                    _ => {}
                }
                // Send device event to the graphics scene
                application.update(ApplicationEvent::from_winit_device_event(&event));
            }
            // Process all the events
            event::Event::MainEventsCleared => {
                window.request_redraw();
            }
            //
            event::Event::RedrawRequested(_) => {
                let frame = swap_chain.get_next_texture().unwrap();
                application.render(&frame.view);
            }
            _ => {}
        }
    });
}
