use crate::client::AppMessage;
use iced::{
    widget::{pin, Container},
    Point, Size, Task,
};

pub trait Message {
    type OwnMessage;
    fn convert_msg(msg: Self::OwnMessage) -> AppMessage;
    fn convert_msg_to_task(msg: Self::OwnMessage) -> Task<AppMessage> {
        Task::done(Self::convert_msg(msg))
    }
    /// This function can handle 4 things:
    /// 1: update_with_msg functions of other ui elements of lesser hierarchy,
    /// 2: arbitrary stuff within the struct (e.g. start animations),
    /// 3: AnimationStarter start functions,
    /// 4: AnimationEndSensor start functions,
    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage>;
}

/// This trait needs the Message trait because animations are supposed to start with the
/// update_with_msg function.
pub trait Animated {
    /// Call this every AnimationTick.
    /// This function can handle 4 things. ALWAYS handle them in this order
    /// (only 2 and 3 can be switched):
    ///
    /// 1: AnimationStarter check functions ->
    /// 2: update_animations functions of other ui elements of lesser hierarchy ->
    /// 3: next_frame functions of animations ->
    /// 4: AnimationEndSensor check functions
    ///
    /// Note: Ensure that 1 and 4 are actually called at 1 and 4 respectively or otherwise there
    ///       will be off by one bugs!
    fn update_animations(&mut self);
}

pub trait Resizable {
    /// Every time an resize event occures call this function.
    /// Use it to set self.window_size and to call other update_size functions of ui elements of
    /// lesser hierarchy
    fn update_size(&mut self, window_size: Size);
    /// Uses the window size from self to calculate the total width of the ui element.
    fn width(&self) -> f32;
    /// Uses the window size from self to calculate the total height of the ui element.
    fn height(&self) -> f32;
}

pub trait SizeFromOutside: Resizable {
    /// Uses a given window size to calculate the width of the total ui element.
    fn width_for(window_size: Size) -> f32;
    /// Uses a given window size to calculate the height of the total ui element.
    fn height_for(window_size: Size) -> f32;
}

pub trait Viewable {
    fn view<'a>(&self) -> Container<'a, AppMessage>;
    fn view_and_move<'a>(&self, x: f32, y: f32) -> Container<'a, AppMessage> {
        Container::new(pin(self.view()).position(Point::new(x, y)))
    }
}
