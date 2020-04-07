use crate::application::*;
use crate::style;

use iced_wgpu::Renderer;
use iced_winit::{checkbox, Checkbox, Column, Container, Element, Length, Space, Text};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    MoleculesCheckboxToggled(bool),
    GridCheckboxToggled(bool),
    AabbsCheckboxToggled(bool),
}

pub struct UserInterface {}

impl UserInterface {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&self, message: Message, application: &mut Application) {
        match message {
            Message::MoleculesCheckboxToggled(b) => {
                application.options.render_molecules = b;
            }
            Message::GridCheckboxToggled(b) => {
                application.options.render_grid = b;
            }
            Message::AabbsCheckboxToggled(b) => {
                application.options.render_aabbs = b;
            }
        }
    }

    pub fn view<'a>(&'a mut self, options: &ApplicationOptions) -> Element<'a, Message, Renderer> {
        Container::new(
            Column::new()
                .push(Text::new("Options"))
                .push(Space::with_height(Length::Units(4)))
                .push(Checkbox::new(
                    options.render_molecules,
                    "Render molecules",
                    Message::MoleculesCheckboxToggled,
                ))
                .push(Space::with_height(Length::Units(4)))
                .push(Checkbox::new(options.render_grid, "Render grid", Message::GridCheckboxToggled))
                .push(Space::with_height(Length::Units(4)))
                .push(Checkbox::new(options.render_aabbs, "Render AABBs", Message::AabbsCheckboxToggled))
                .push(Space::with_height(Length::Units(4)))
                .padding(12),
        )
        .style(style::Theme::Dark)
        .into()
    }
}
