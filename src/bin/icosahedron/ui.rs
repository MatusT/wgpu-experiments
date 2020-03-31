use crate::application::*;

use iced_wgpu::Renderer;
use iced_winit::{container, slider, Align, Background, Color, Column, Container, Element, Length, Row, Slider, Text};

pub struct UserInterface {
    slider: slider::State,
}

#[derive(Debug)]
pub enum Message {
    NumberCubesChanges(f32),
}

impl UserInterface {
    pub fn new() -> Self {
        Self {
            slider: iced_wgpu::slider::State::new(),
        }
    }

    pub fn update(&self, message: Message, application: &mut Application) {
        match message {
            Message::NumberCubesChanges(n) => {
                let mut positions = Vec::new();
                let n_i32 = n as i32;
                for x in -n_i32 / 2..n_i32 / 2 {
                    for y in -n_i32 / 2..n_i32 / 2 {
                        for z in -n_i32 / 2..n_i32 / 2 {
                            positions.push(x as f32);
                            positions.push(y as f32);
                            positions.push(z as f32);
                            positions.push(0.0);
                        }
                    }
                }

                application.positions_instanced_buffer = application
                    .device
                    .create_buffer_mapped::<f32>(positions.len(), wgpu::BufferUsage::STORAGE_READ)
                    .fill_from_slice(&positions);

                application.bind_group = application.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &application.pipeline.bind_group_layout,
                    bindings: &[
                        wgpu::Binding {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer {
                                buffer: &application.camera_buffer,
                                range: 0..192,
                            },
                        },
                        wgpu::Binding {
                            binding: 1,
                            resource: wgpu::BindingResource::Buffer {
                                buffer: &application.positions_instanced_buffer,
                                range: 0..(positions.len() * std::mem::size_of::<f32>()) as u64,
                            },
                        },
                    ],
                });

                application.options.n = n as i32;
            }
        }
    }

    pub fn view<'a>(&'a mut self, options: &ApplicationOptions) -> Element<'a, Message, Renderer> {
        let slider = &mut self.slider;

        let sliders =
            Row::new()
                .width(Length::Units(500))
                .spacing(20)
                .push(Slider::new(slider, 0.0..=1000.0, options.n as f32, move |n| {
                    Message::NumberCubesChanges(n)
                }));

        Column::new()
            .push(
                Container::new(
                    Row::new()
                        .width(Length::Units(200))
                        .height(Length::Fill)
                        .align_items(Align::Start)
                        .push(
                            Column::new().width(Length::Fill).align_items(Align::End).push(
                                Column::new()
                                    .padding(10)
                                    .spacing(10)
                                    .push(Text::new("Background color").color(Color::WHITE))
                                    .push(sliders)
                                    .push(Text::new(format!("{:?}", options.n)).size(14).color(Color::WHITE)),
                            ),
                        ),
                )
                .style(style::Theme::Dark),
            )
            .padding(28)
            .into()
    }
}

mod style {
    use iced::{button, checkbox, container, progress_bar, radio, scrollable, slider, text_input};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Theme {
        Light,
        Dark,
    }

    impl Theme {
        pub const ALL: [Theme; 2] = [Theme::Light, Theme::Dark];
    }

    impl Default for Theme {
        fn default() -> Theme {
            Theme::Light
        }
    }

    impl From<Theme> for Box<dyn container::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Container.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn radio::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Radio.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn text_input::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::TextInput.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn button::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => light::Button.into(),
                Theme::Dark => dark::Button.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn scrollable::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Scrollable.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn slider::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Slider.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn progress_bar::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::ProgressBar.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn checkbox::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Checkbox.into(),
            }
        }
    }

    mod light {
        use iced::{button, Background, Color, Vector};

        pub struct Button;

        impl button::StyleSheet for Button {
            fn active(&self) -> button::Style {
                button::Style {
                    background: Some(Background::Color(Color::from_rgb(0.11, 0.42, 0.87))),
                    border_radius: 12,
                    shadow_offset: Vector::new(1.0, 1.0),
                    text_color: Color::from_rgb8(0xEE, 0xEE, 0xEE),
                    ..button::Style::default()
                }
            }

            fn hovered(&self) -> button::Style {
                button::Style {
                    text_color: Color::WHITE,
                    shadow_offset: Vector::new(1.0, 2.0),
                    ..self.active()
                }
            }
        }
    }

    mod dark {
        use iced::{button, checkbox, container, progress_bar, radio, scrollable, slider, text_input, Background, Color};

        const SURFACE: Color = Color::from_rgb(0x40 as f32 / 255.0, 0x44 as f32 / 255.0, 0x4B as f32 / 255.0);

        const ACCENT: Color = Color::from_rgb(0x6F as f32 / 255.0, 0xFF as f32 / 255.0, 0xE9 as f32 / 255.0);

        const ACTIVE: Color = Color::from_rgb(0x72 as f32 / 255.0, 0x89 as f32 / 255.0, 0xDA as f32 / 255.0);

        const HOVERED: Color = Color::from_rgb(0x67 as f32 / 255.0, 0x7B as f32 / 255.0, 0xC4 as f32 / 255.0);

        pub struct Container;

        impl container::StyleSheet for Container {
            fn style(&self) -> container::Style {
                container::Style {
                    background: Some(Background::Color(Color::from_rgb8(0x36, 0x39, 0x3F))),
                    text_color: Some(Color::WHITE),
                    border_radius: 3,
                    ..container::Style::default()
                }
            }
        }

        pub struct Radio;

        impl radio::StyleSheet for Radio {
            fn active(&self) -> radio::Style {
                radio::Style {
                    background: Background::Color(SURFACE),
                    dot_color: ACTIVE,
                    border_width: 1,
                    border_color: ACTIVE,
                }
            }

            fn hovered(&self) -> radio::Style {
                radio::Style {
                    background: Background::Color(Color { a: 0.5, ..SURFACE }),
                    ..self.active()
                }
            }
        }

        pub struct TextInput;

        impl text_input::StyleSheet for TextInput {
            fn active(&self) -> text_input::Style {
                text_input::Style {
                    background: Background::Color(SURFACE),
                    border_radius: 2,
                    border_width: 0,
                    border_color: Color::TRANSPARENT,
                }
            }

            fn focused(&self) -> text_input::Style {
                text_input::Style {
                    border_width: 1,
                    border_color: ACCENT,
                    ..self.active()
                }
            }

            fn hovered(&self) -> text_input::Style {
                text_input::Style {
                    border_width: 1,
                    border_color: Color { a: 0.3, ..ACCENT },
                    ..self.focused()
                }
            }

            fn placeholder_color(&self) -> Color {
                Color::from_rgb(0.4, 0.4, 0.4)
            }

            fn value_color(&self) -> Color {
                Color::WHITE
            }
        }

        pub struct Button;

        impl button::StyleSheet for Button {
            fn active(&self) -> button::Style {
                button::Style {
                    background: Some(Background::Color(ACTIVE)),
                    border_radius: 3,
                    text_color: Color::WHITE,
                    ..button::Style::default()
                }
            }

            fn hovered(&self) -> button::Style {
                button::Style {
                    background: Some(Background::Color(HOVERED)),
                    text_color: Color::WHITE,
                    ..self.active()
                }
            }

            fn pressed(&self) -> button::Style {
                button::Style {
                    border_width: 1,
                    border_color: Color::WHITE,
                    ..self.hovered()
                }
            }
        }

        pub struct Scrollable;

        impl scrollable::StyleSheet for Scrollable {
            fn active(&self) -> scrollable::Scrollbar {
                scrollable::Scrollbar {
                    background: Some(Background::Color(SURFACE)),
                    border_radius: 2,
                    border_width: 0,
                    border_color: Color::TRANSPARENT,
                    scroller: scrollable::Scroller {
                        color: ACTIVE,
                        border_radius: 2,
                        border_width: 0,
                        border_color: Color::TRANSPARENT,
                    },
                }
            }

            fn hovered(&self) -> scrollable::Scrollbar {
                let active = self.active();

                scrollable::Scrollbar {
                    background: Some(Background::Color(Color { a: 0.5, ..SURFACE })),
                    scroller: scrollable::Scroller {
                        color: HOVERED,
                        ..active.scroller
                    },
                    ..active
                }
            }

            fn dragging(&self) -> scrollable::Scrollbar {
                let hovered = self.hovered();

                scrollable::Scrollbar {
                    scroller: scrollable::Scroller {
                        color: Color::from_rgb(0.85, 0.85, 0.85),
                        ..hovered.scroller
                    },
                    ..hovered
                }
            }
        }

        pub struct Slider;

        impl slider::StyleSheet for Slider {
            fn active(&self) -> slider::Style {
                slider::Style {
                    rail_colors: (ACTIVE, Color { a: 0.1, ..ACTIVE }),
                    handle: slider::Handle {
                        shape: slider::HandleShape::Circle { radius: 9 },
                        color: ACTIVE,
                        border_width: 0,
                        border_color: Color::TRANSPARENT,
                    },
                }
            }

            fn hovered(&self) -> slider::Style {
                let active = self.active();

                slider::Style {
                    handle: slider::Handle {
                        color: HOVERED,
                        ..active.handle
                    },
                    ..active
                }
            }

            fn dragging(&self) -> slider::Style {
                let active = self.active();

                slider::Style {
                    handle: slider::Handle {
                        color: Color::from_rgb(0.85, 0.85, 0.85),
                        ..active.handle
                    },
                    ..active
                }
            }
        }

        pub struct ProgressBar;

        impl progress_bar::StyleSheet for ProgressBar {
            fn style(&self) -> progress_bar::Style {
                progress_bar::Style {
                    background: Background::Color(SURFACE),
                    bar: Background::Color(ACTIVE),
                    border_radius: 10,
                }
            }
        }

        pub struct Checkbox;

        impl checkbox::StyleSheet for Checkbox {
            fn active(&self, is_checked: bool) -> checkbox::Style {
                checkbox::Style {
                    background: Background::Color(if is_checked { ACTIVE } else { SURFACE }),
                    checkmark_color: Color::WHITE,
                    border_radius: 2,
                    border_width: 1,
                    border_color: ACTIVE,
                }
            }

            fn hovered(&self, is_checked: bool) -> checkbox::Style {
                checkbox::Style {
                    background: Background::Color(Color {
                        a: 0.8,
                        ..if is_checked { ACTIVE } else { SURFACE }
                    }),
                    ..self.active(is_checked)
                }
            }
        }
    }
}
