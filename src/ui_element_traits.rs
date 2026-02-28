use crate::client::AppMessage;
use iced::{
    widget::{pin, Container},
    Point, Size, Task,
};

pub trait Notifiable {
    /// The OwnMessage needs the Message trait or update_with_msg can't return Task<AppMessage>
    type OwnMessage: Message;
    /// This function can handle 2 things:
    /// - update_with_msg functions of other ui elements of lesser hierarchy,
    /// - arbitrary stuff within the struct (e.g. start animations with AnimationStarter)
    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage>;
}

/// This trait needs to be implemented for every animated ui element.
pub trait Animated {
    /// Call this every AnimationTick.
    /// This function can handle 3 things:
    ///
    /// - AnimationStarter check functions
    /// - update_animations functions of other ui elements of lesser hierarchy
    /// - next_frame functions of animations
    fn update_animations(&mut self) -> Task<AppMessage>;
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

/// Implement this if you want to know the measures of an ui_element from elsewhere.
/// This comes in handy when the size of the parent ui element is defined by the child ui element.
pub trait SizeFromOutside: Resizable {
    /// Uses a given window size to calculate the width of the total ui element.
    fn width_for(window_size: Size) -> f32;
    /// Uses a given window size to calculate the height of the total ui element.
    fn height_for(window_size: Size) -> f32;
}

/// Like Resizable, but without the height.
/// Implmenet this instead if the item can dynamically interference its height.
pub trait ResizableDynHeight {
    /// Every time an resize event occures call this function.
    /// Use it to set self.window_size and to call other update_size functions of ui elements of
    /// lesser hierarchy
    fn update_size(&mut self, window_size: Size);
    /// Uses the window size from self to calculate the total width of the ui element.
    fn width(&self) -> f32;
}

pub trait Viewable {
    fn view<'a>(&self) -> Container<'a, AppMessage>;
    fn view_and_move<'a>(&self, x: f32, y: f32) -> Container<'a, AppMessage> {
        Container::new(pin(self.view()).position(Point::new(x, y)))
    }
}

/// This trait is for child message enums of AppMessage to convert them to Task or AppMessage.
/// You need to implement convert_msg_from by using clone to use all other methods.
pub trait Message: Clone {
    fn convert_msg_from(msg: Self) -> AppMessage;
    fn convert_msg(&self) -> AppMessage {
        Self::convert_msg_from(self.clone())
    }
    fn convert_msg_to_task(&self) -> Task<AppMessage> {
        Task::done(self.convert_msg())
    }
}

/// Trait to check if the message has an usize at the end and replaces it if its exists.
/// This is only needed if the message is used to start an animation via AnimationStarter
/// where you have to iterate the message.
pub trait ReplaceUsize: Message {
    fn replace_usize(&self, value: usize) -> Self;
}
