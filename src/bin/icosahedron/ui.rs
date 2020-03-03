use crate::application::ApplicationOptions;

use iced_wgpu::Renderer;
use iced_winit::{slider, Align, Color, Column, Element, Length, Row, Slider, Text};

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
            slider: slider::State::new(),
        }
    }

    pub fn update(&self, message: Message, options: &mut ApplicationOptions) {
        match message {
            Message::NumberCubesChanges(n) => {
                options.n = n as i32;
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

        Row::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Align::End)
            .push(
                Column::new().width(Length::Fill).align_items(Align::End).push(
                    Column::new()
                        .padding(10)
                        .spacing(10)
                        .push(Text::new("Background color").color(Color::WHITE))
                        .push(sliders)
                        .push(Text::new(format!("{:?}", options.n)).size(14).color(Color::WHITE)),
                ),
            )
            .into()
    }
}
