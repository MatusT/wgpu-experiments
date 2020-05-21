use crate::application::*;
use crate::style;

use iced_wgpu::Renderer;
use iced_winit::{Checkbox, Column, Container, Element, Length, Space, Text};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    DepthPrepassCheckboxToggled(bool),
    AabbsCheckboxToggled(bool),
    OutputCheckboxToggled(bool),
}

pub struct UserInterface {}

impl UserInterface {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&self, message: Message, application: &mut Application) {
        match message {
            Message::DepthPrepassCheckboxToggled(b) => {
                application.options.render_depth_prepass = b;
            }
            Message::AabbsCheckboxToggled(b) => {
                application.options.render_aabbs = b;
            }
            Message::OutputCheckboxToggled(b) => {
                application.options.render_output = b;
            }
        }
    }

    pub fn view<'a>(&'a mut self, options: &ApplicationOptions) -> Element<'a, Message, Renderer> {
        Container::new(
            Column::new()
                .push(Text::new("Options"))
                .push(Space::with_height(Length::Units(4)))
                .push(Checkbox::new(
                    options.render_depth_prepass,
                    "Depth prepass",
                    Message::DepthPrepassCheckboxToggled,
                ))
                .push(Space::with_height(Length::Units(4)))
                .push(Checkbox::new(options.render_aabbs, "AABBs", Message::AabbsCheckboxToggled))
                .push(Space::with_height(Length::Units(4)))
                .push(Checkbox::new(options.render_output, "Output", Message::OutputCheckboxToggled))
                .push(Space::with_height(Length::Units(4)))
                .padding(12),
        )
        .style(style::Theme::Dark)
        .into()
    }
}
