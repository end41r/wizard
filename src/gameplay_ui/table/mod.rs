pub mod middle;

use crate::{
    client::AppMessage,
    gameplay_ui::{
        hand::HandMessage,
        table::middle::{TableMiddleMessage, ViewableTableMiddle},
    },
    ui_element_traits::*,
};
use iced::{widget::Container, Size, Task};

#[derive(Debug, Clone)]
pub enum TableMessage {
    TableMiddleMessage(TableMiddleMessage),
}

impl Message for TableMessage {
    fn convert_msg_from(msg: Self) -> AppMessage {
        AppMessage::TableMessage(msg)
    }
}

pub struct ViewableTable {
    window_size: Size,
    middle: ViewableTableMiddle,
}

impl ViewableTable {
    pub fn new(window_size: Size) -> Self {
        Self {
            window_size,
            middle: ViewableTableMiddle::new(window_size),
        }
    }
}

impl Notifiable for ViewableTable {
    type OwnMessage = TableMessage;

    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage> {
        match msg {
            TableMessage::TableMiddleMessage(table_middle_msg) => {
                self.middle.update_with_msg(table_middle_msg)
            }
        }
    }
}

impl Animated for ViewableTable {
    fn update_animations(&mut self) -> Task<AppMessage> {
        self.middle.update_animations()
    }
}

impl Resizable for ViewableTable {
    fn heigth(&self) -> f32 {
        self.middle.heigth()
    }
    fn width(&self) -> f32 {
        self.middle.width()
    }
    fn update_size(&mut self, window_size: iced::Size) {
        self.window_size = window_size;
        self.middle.update_size(window_size);
    }
}

impl Viewable for ViewableTable {
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        self.middle.view()
    }
}
