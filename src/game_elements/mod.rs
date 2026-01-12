pub mod hand;
pub mod hand_card;

use iced::{Size, widget::Container};
use crate::client::AppMessage;

pub trait GameElement {
    type HigherMessage;
    type OwnMessage;
    fn convert_msg(msg: Self::HigherMessage) -> Self::OwnMessage;
    fn convert_to_app_message(msg: Self::OwnMessage) -> AppMessage;
    fn update_with_msg(&mut self, msg: Self::OwnMessage);
    fn update_animations(&mut self);
    fn update_size(&mut self, window_size: Size);
    fn view<'a>(&self) -> Container<'a, AppMessage>;
    fn view_and_move<'a>(&self, x: f32, y: f32) -> Container<'a, AppMessage>;
}