use crate::client::AppMessage;
use iced::{Point, Size, widget::{Container, pin}};

pub trait Message {
    type OwnMessage;
    fn convert_to_app_message(msg: Self::OwnMessage) -> AppMessage;
    /// Convey the msg to lower GameElements asap (if they exist) before doing anything else.
    fn update_with_msg(&mut self, msg: Self::OwnMessage);
}

pub trait Animated: Message {
    /// Call this every AnimationTick.
    /// First call other update_animations then animation tickers.
    fn update_animations(&mut self);
}

pub trait Resizable {
    fn update_size(&mut self, window_size: Size);
}

pub trait Viewable {
    fn view<'a>(&self) -> Container<'a, AppMessage>;
    fn view_and_move<'a>(&self, x: f32, y: f32) -> Container<'a, AppMessage> {
        Container::new(pin(self.view()).position(Point::new(x, y)))
    }
}
