use iced::{
    widget::{image, pin, Container},
    Point, Size, Task,
};

use crate::{
    api::{Avatar, AvatarKind},
    client::AppMessage,
    gameplay_ui::{
        table::TableMessage, AVATAR_IMG_SIZE_MULT_WITH_WINDOW_WIDTH,
        AVATAR_SIZE_MULT_WTIH_WINDOW_WIDTH,
    },
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
pub struct ViewableAvatar {
    window_size: Size,
    avatar: Avatar,
}

impl ViewableAvatar {
    pub fn new(window_size: Size, avatar_kind: AvatarKind) -> Self {
        Self {
            window_size,
            avatar: avatar_kind.to_avatar(),
        }
    }
    fn avatar_img_position(&self) -> Point {
        let size: f32 = self.width()
            * (AVATAR_SIZE_MULT_WTIH_WINDOW_WIDTH / AVATAR_IMG_SIZE_MULT_WITH_WINDOW_WIDTH - 1.0)
            / 2.0;
        Point::new(size, size)
    }
}

impl Notifiable for ViewableAvatar {
    type OwnMessage = AvatarMessage;
    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage> {
        Task::none()
    }
}

impl Animated for ViewableAvatar {
    fn update_animations(&mut self) -> Task<AppMessage> {
        Task::none()
    }
}

impl Resizable for ViewableAvatar {
    fn update_size(&mut self, window_size: Size) {
        self.window_size = window_size;
    }
    fn width(&self) -> f32 {
        self.window_size.width * AVATAR_SIZE_MULT_WTIH_WINDOW_WIDTH
    }
    fn height(&self) -> f32 {
        self.width()
    }
}

impl Viewable for ViewableAvatar {
    fn view<'a>(&self) -> iced::widget::Container<'a, AppMessage> {
        Container::new(
            pin(image(self.avatar.img_path())
                .width(AVATAR_IMG_SIZE_MULT_WITH_WINDOW_WIDTH * self.window_size.width)
                .height(AVATAR_IMG_SIZE_MULT_WITH_WINDOW_WIDTH * self.window_size.width))
            .position(self.avatar_img_position()),
        )
        .width(self.width())
        .height(self.height())
    }
}
