use iced::{widget::Container, Size, Task};

use crate::{
    client::AppMessage,
    gameplay_ui::table::TableMessage,
    ui_element_traits::{Animated, Message, Notifiable, Resizable, Viewable},
};

#[derive(Clone, Debug)]
pub enum AvatarMessage {}

impl Message for AvatarMessage {
    fn convert_msg_from(msg: Self) -> crate::client::AppMessage {
        TableMessage::convert_msg_from(TableMessage::AvatarMessage(msg))
    }
}

#[derive(Clone, Debug)]
pub struct Avatar {
    window_size: Size,
}

impl Avatar {
    pub fn new(window_size: Size) -> Self {
        Self { window_size }
    }
}

impl Notifiable for Avatar {
    type OwnMessage = AvatarMessage;
    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage> {
        Task::none()
    }
}

impl Animated for Avatar {
    fn update_animations(&mut self) -> Task<AppMessage> {
        Task::none()
    }
}

impl Resizable for Avatar {
    fn update_size(&mut self, window_size: Size) {
        self.window_size = window_size;
    }
    fn width(&self) -> f32 {
        0.0
    }
    fn height(&self) -> f32 {
        self.width()
    }
}

impl Viewable for Avatar {
    fn view<'a>(&self) -> iced::widget::Container<'a, AppMessage> {
        Container::new("placeholder")
    }
}
