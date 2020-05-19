use crate::application::*;

use iced_wgpu::Renderer;
use iced_winit::{Column, Container, Element, Text};

#[derive(Debug, Clone, Copy)]
pub enum Message {}

pub struct UserInterface {}

impl UserInterface {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&self, _: Message, _: &mut Application) {}

    pub fn view<'a>(&'a mut self, _: &ApplicationOptions) -> Element<'a, Message, Renderer> {
        Container::new(Column::new().push(Text::new("Options")).padding(12)).into()
    }
}
