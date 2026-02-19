use std::f32::consts::PI;

use iced::{
    widget::{image, pin, stack, Container},
    Point, Size, Task,
};

use derive_more::{Deref, DerefMut};

use crate::{
    animation::{BasicAnimation, CircularAnimation, Easing, ReversableBasicAnimation},
    api::{Avatar, AvatarKind, CARD_BACK_PATH},
    client::{AppMessage, TaskBatcher},
    gameplay_ui::{
        table::TableMessage, AVATAR_CARD_SIZE_MULT_WITH_WINDOW_WIDTH,
        AVATAR_IMG_SIZE_MULT_WITH_WINDOW_WIDTH, AVATAR_SIZE_MULT_WTIH_WINDOW_WIDTH,
    },
    ui_element_traits::{Animated, Message, Notifiable, Resizable, Viewable},
};

#[derive(Clone, Debug)]
pub enum AvatarMessage {
    AddCards(usize),
    PlayCard,
}

impl Message for AvatarMessage {
    fn convert_msg_from(msg: Self) -> crate::client::AppMessage {
        TableMessage::convert_msg_from(TableMessage::AvatarMessage(msg))
    }
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct SpriteAnimation(CircularAnimation);

impl SpriteAnimation {
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

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct RevealAnimation(ReversableBasicAnimation);

impl RevealAnimation {
    fn new() -> Self {
        Self(ReversableBasicAnimation::new(100))
    }
    fn get_opacity(&self) -> f32 {
        1.0 - self.progress(Easing::OutElastic)
    }
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct PlayCardAnimation(BasicAnimation);

impl PlayCardAnimation {
    fn new() -> Self {
        Self(BasicAnimation::new(100))
    }
    fn get_opacity(&self) -> f32 {
        self.progress(Easing::OutCubic)
    }
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct CardRotationAnimation(CircularAnimation);

impl CardRotationAnimation {
    fn new() -> Self {
        Self(CircularAnimation::new(400))
    }
    /// Scaled to PI not to a 100%.
    fn get_rotation(&self) -> f32 {
        self.progress(Easing::Linear) * 2.0 * PI
    }
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct InterferenceAnimation(BasicAnimation);

impl InterferenceAnimation {
    fn new() -> Self {
        Self(BasicAnimation::new(100))
    }
    fn get_progress(&self) -> f32 {
        self.progress(Easing::OutSine)
    }
}

#[derive(Clone, Debug)]
pub struct ViewableAvatar {
    window_size: Size,
    avatar: Avatar,
    cards: usize,
    interference: bool,
    sprite_animation: SpriteAnimation,
    reveal_animation: RevealAnimation,
    play_card_animation: PlayCardAnimation,
    card_rotation_animation: CardRotationAnimation,
    interference_animation: InterferenceAnimation,
}

impl ViewableAvatar {
    pub fn new(window_size: Size, avatar_kind: AvatarKind) -> Self {
        let mut viewable_avatar = Self {
            window_size,
            avatar: avatar_kind.to_avatar(),
            cards: 20,
            interference: false,
            sprite_animation: SpriteAnimation::new(),
            reveal_animation: RevealAnimation::new(),
            play_card_animation: PlayCardAnimation::new(),
            card_rotation_animation: CardRotationAnimation::new(),
            interference_animation: InterferenceAnimation::new(),
        };
        viewable_avatar.sprite_animation.start();
        viewable_avatar.card_rotation_animation.start();
        viewable_avatar
    }
    fn avatar_img_position(&self) -> Point {
        let size: f32 = self.window_size.width * AVATAR_CARD_SIZE_MULT_WITH_WINDOW_WIDTH;
        Point::new(size, size)
    }
    fn card_position(&self, card_number: usize) -> Point {
        let position: Point = self.card_position_helper(card_number);
        if self.interference {
            let position_with_less: Point = self.card_position_helper(card_number - 1);
            let position_offset: f32 = self.interference_animation.get_progress()
                * (position.x - position_with_less.x).abs();
            Point::new(position.x + position_offset, position.y + position_offset)
        } else {
            position
        }
    }
    fn card_position_helper(&self, card_number: usize) -> Point {
        let card_size: f32 = self.window_size.width * AVATAR_CARD_SIZE_MULT_WITH_WINDOW_WIDTH;
        let circle_radius: f32 = self.width() / 2.0;
        let mut x: f32 = circle_radius - card_size / 2.0;
        let mut y: f32 = x;
        // The angle is scaled from 0.0 to PI, not from 0.0 to 1.0.
        let rotation_angle: f32 = ((card_number as f32 / self.cards as f32) * 2.0 * PI
            + self.card_rotation_animation.get_rotation())
            - (PI / 2.0); // To start on top of the circle.
        x += x * rotation_angle.cos();
        y += y * rotation_angle.sin();
        Point::new(x, y)
    }
    fn card_rotation(&self, card_number: usize) -> f32 {
        let rotation = self.card_rotation_helper(card_number);
        if self.interference {
            let rotation_with_less: f32 = self.card_rotation_helper(card_number - 1);
            rotation
                + self.interference_animation.get_progress() * (rotation - rotation_with_less).abs()
        } else {
            rotation
        }
    }
    fn card_rotation_helper(&self, card_number: usize) -> f32 {
        (card_number as f32 / self.cards as f32) * 2.0 * PI
            + self.card_rotation_animation.get_rotation()
    }
}

impl Notifiable for ViewableAvatar {
    type OwnMessage = AvatarMessage;
    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage> {
        match msg {
            AvatarMessage::AddCards(cards) => {
                self.cards = cards;
                self.reveal_animation.start_force();
            }
            AvatarMessage::PlayCard => {
                if self.cards > 0 {
                    self.cards -= 1;
                    self.play_card_animation.start_force();
                    self.sprite_animation.start_force();
                    self.avatar.start_casting();
                }
                if self.cards > 1 {
                    self.interference = true;
                    self.interference_animation.start_force();
                }
            }
        }
        Task::none()
    }
}

impl Animated for ViewableAvatar {
    fn update_animations(&mut self) -> Task<AppMessage> {
        let mut tb = TaskBatcher::new();

        tb.push(self.sprite_animation.next_frame());
        if !self.avatar.is_casting() && self.sprite_animation.new_frame() {
            self.avatar.next_pose();
        } else if self.avatar.is_casting() && self.sprite_animation.new_casting_frame() {
            self.avatar.next_pose();
        };

        tb.push(self.reveal_animation.next_frame());
        tb.push(self.play_card_animation.next_frame());
        tb.push(self.card_rotation_animation.next_frame());
        tb.push(self.interference_animation.next_frame());
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
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let mut avatar = stack!();
        let sprite_size = AVATAR_IMG_SIZE_MULT_WITH_WINDOW_WIDTH * self.window_size.width;
        let sprite = pin(image(self.avatar.img_path())
            .width(sprite_size)
            .height(sprite_size))
        .position(self.avatar_img_position());
        avatar = avatar.push(sprite);
        if self.cards > 0 {
            let play_opacity: f32 = self.play_card_animation.get_opacity();
            let card_size: f32 = self.window_size.width * AVATAR_CARD_SIZE_MULT_WITH_WINDOW_WIDTH;
            for card in 0..self.cards {
                let opacity: f32 = if card == self.cards {
                    self.reveal_animation.get_opacity().min(play_opacity)
                } else {
                    self.reveal_animation.get_opacity()
                };
                avatar = avatar.push(
                    pin(image(CARD_BACK_PATH)
                        .rotation(self.card_rotation(card))
                        .scale(0.8)
                        .opacity(opacity)
                        .width(card_size)
                        .height(card_size))
                    .position(self.card_position(card)),
                );
            }
        }
        Container::new(avatar)
            .width(self.width())
            .height(self.height())
    }
}
