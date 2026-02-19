use iced::{
    widget::{image, pin, Container},
    Point, Size, Task,
};

use derive_more::{Deref, DerefMut};

use crate::{
    animation::CircularAnimation,
    api::{Avatar, AvatarKind},
    client::{AppMessage, TaskBatcher},
    gameplay_ui::{
        table::TableMessage, AVATAR_IMG_SIZE_MULT_WITH_WINDOW_WIDTH,
        AVATAR_SIZE_MULT_WTIH_WINDOW_WIDTH,
    },
    ui_element_traits::{Animated, Message, Notifiable, Resizable, Viewable},
};

#[derive(Clone, Debug)]
pub enum AvatarMessage {
    Cast,
}

impl Message for AvatarMessage {
    fn convert_msg_from(msg: Self) -> crate::client::AppMessage {
        TableMessage::convert_msg_from(TableMessage::AvatarMessage(msg))
    }
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct MainLoopAnimation(CircularAnimation);

impl MainLoopAnimation {
    fn new() -> Self {
        Self(CircularAnimation::new(100))
    }
    fn new_frame(&self) -> bool {
        self.current_frame_number() == 80 || self.current_frame_number() == self.max_frame_number()
    }
    fn new_casting_frame(&self) -> bool {
        self.current_frame_number() == 25
            || self.current_frame_number() == 50
            || self.current_frame_number() == 75
            || self.current_frame_number() == self.max_frame_number()
    }
}

#[derive(Clone, Debug)]
pub struct ViewableAvatar {
    window_size: Size,
    avatar: Avatar,
    main_loop_animation: MainLoopAnimation,
}

impl ViewableAvatar {
    pub fn new(window_size: Size, avatar_kind: AvatarKind) -> Self {
        let mut viewable_avatar = Self {
            window_size,
            avatar: avatar_kind.to_avatar(),
            main_loop_animation: MainLoopAnimation::new(),
        };
        viewable_avatar.main_loop_animation.start();
        viewable_avatar
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
        match msg {
            AvatarMessage::Cast => {
                self.main_loop_animation.start_force();
                self.avatar.start_casting();
            }
        }
        Task::none()
    }
}

impl Animated for ViewableAvatar {
    fn update_animations(&mut self) -> Task<AppMessage> {
        let mut tb = TaskBatcher::new();
        tb.push(self.main_loop_animation.next_frame());
        if !self.avatar.is_casting() && self.main_loop_animation.new_frame() {
            self.avatar.next_pose();
        } else if self.avatar.is_casting() && self.main_loop_animation.new_casting_frame() {
            self.avatar.next_pose();
        };
        tb.batch()
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
