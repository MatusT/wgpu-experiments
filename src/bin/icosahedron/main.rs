mod application;
mod ui;

use iced_wgpu::{wgpu, window::SwapChain, Primitive, Renderer, Settings, Target, Viewport};
use iced_winit::{winit, Cache, Clipboard, MouseCursor, Size, UserInterface};
use wgpu_experiments::{ApplicationEvent, ApplicationSkeleton};

fn main() {
    use winit::{
        event,
        event::DeviceEvent,
        event::ModifiersState,
        event::WindowEvent,
        event_loop::{ControlFlow, EventLoop},
    };

    // Initialize winit
    let event_loop = EventLoop::new();

    let (window, size, surface) = {
        let window = winit::window::Window::new(&event_loop).unwrap();
        window.set_title("Hello world");
        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);
        (window, size, surface)
    };

    let mut modifiers = ModifiersState::default();

    // Initialize the graphics scene
    let mut application = application::Application::new(size.width, size.height);

    // Initialize swapchain
    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Vsync,
    };
    let mut swap_chain = application.device().create_swap_chain(&surface, &sc_desc);

    // Initialize GUI
    let mut events = Vec::new();
    let mut cache = Some(Cache::default());
    let mut renderer = Renderer::new(application.device_mut(), Settings::default());
    let mut output = (Primitive::None, MouseCursor::OutOfBounds);
    let clipboard = Clipboard::new(&window);
    let mut ui = ui::UserInterface::new();

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
                    _ => {}
                };

                let event = event.to_static().unwrap();

                // Send window event to the graphics scene
                application.update(ApplicationEvent::from_winit_window_event(&event));

                // Map window event to iced event
                if let Some(event) = iced_winit::conversion::window_event(event, window.scale_factor(), modifiers) {
                    events.push(event);
                }                
            }
            event::Event::DeviceEvent { event, .. } => {
                match event {
                    DeviceEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = new_modifiers;
                    }
                    _ => {}
                }
                // Send device event to the graphics scene
                application.update(ApplicationEvent::from_winit_device_event(&event));
            }
            // Process all the events
            event::Event::MainEventsCleared => {
                // We need to:
                // 1. Process events of our user interface.
                // 2. Update state as a result of any interaction.
                // 3. Generate a new output for our renderer.

                // First, we build our user interface.
                let mut user_interface = UserInterface::build(
                    ui.view(&application.options()),
                    Size::new(sc_desc.width as f32, sc_desc.height as f32),
                    cache.take().unwrap(),
                    &mut renderer,
                );

                // Then, we process the events, obtaining messages in return.
                let messages = user_interface.update(events.drain(..), clipboard.as_ref().map(|c| c as _), &renderer);

                let user_interface = if messages.is_empty() {
                    // If there are no messages, no interactions we care about have
                    // happened. We can simply leave our user interface as it is.
                    user_interface
                } else {
                    // If there are messages, we need to update our state
                    // accordingly and rebuild our user interface.
                    // We can only do this if we drop our user interface first
                    // by turning it into its cache.
                    cache = Some(user_interface.into_cache());

                    // In this example, `Controls` is the only part that cares
                    // about messages, so updating our state is pretty
                    // straightforward.
                    for message in messages {
                        ui.update(message, &mut application.options_mut());
                    }

                    // Once the state has been changed, we rebuild our updated
                    // user interface.
                    UserInterface::build(
                        ui.view(&application.options()),
                        Size::new(sc_desc.width as f32, sc_desc.height as f32),
                        cache.take().unwrap(),
                        &mut renderer,
                    )
                };

                // Finally, we just need to draw a new output for our renderer,
                output = user_interface.draw(&mut renderer);

                // update our cache,
                cache = Some(user_interface.into_cache());

                // and request a redraw
                window.request_redraw();
            }
            //
            event::Event::RedrawRequested(_) => {
                let frame = swap_chain.get_next_texture();
                // .expect("Timeout when acquiring next swap chain texture");

                // if resized {
                //     let size = window.inner_size();

                //     swap_chain = SwapChain::new(
                //         &device,
                //         &surface,
                //         format,
                //         size.width,
                //         size.height,
                //     );
                // }

                // let (frame, viewport) = swap_chain.next_frame();

                // We draw the scene first
                application.render(&frame.view);

                // And then iced on top
                let mut encoder = application
                    .device()
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
                let viewport = Viewport::new(sc_desc.width, sc_desc.height);
                let mouse_cursor = renderer.draw(
                    application.device_mut(),
                    &mut encoder,
                    Target {
                        texture: &frame.view,
                        viewport: &viewport,
                    },
                    &output,
                    window.scale_factor(),
                    &["Some debug information!"],
                );

                // Then we submit the work
                application.queue_mut().submit(&[encoder.finish()]);

                // And update the mouse cursor
                window.set_cursor_icon(iced_winit::conversion::mouse_cursor(mouse_cursor));
            }
            _ => {}
        }
    });
}
