use std::f32::consts::PI;

use iced::{
    widget::{container, image, image::FilterMethod, pin, stack, text, Container, Pin},
    Alignment, Color, Point, Size, Task,
};

use derive_more::{Deref, DerefMut};

use crate::{
    animation::{BasicAnimation, CircularAnimation, Easing, ReversableBasicAnimation},
    api::{Avatar, AvatarKind, AvatarPose, PlayerId},
    client::{AppMessage, TaskBatcher},
    gameplay_ui::{
        table::TableMessage, AVATAR_FRAME_WIDTH_HEIGHT_RATIO,
        AVATAR_IMG_SIZE_MULT_WITH_WINDOW_WIDTH, AVATAR_SHARD_SIZE_MULT_WITH_WINDOW_WIDTH,
        AVATAR_SIZE_MULT_WITH_WINDOW_WIDTH,
    },
    ui_element_traits::{Animated, Message, Notifiable, Resizable, SizeFromOutside, Viewable},
};

#[derive(Clone, Debug)]
pub enum AvatarMessage {
    AddShards(PlayerId, usize),
    PlayShard(PlayerId),
    InterpolationEnded(PlayerId),
    ChangeTurn(PlayerId),
    NobodiesTurn,
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
        Self(CircularAnimation::new(100, false))
    }
    fn new_frame(&self) -> bool {
        self.current_frame_number() == 80 || self.current_frame_number() == self.max_frame_number()
    }
    fn new_casting_frame(&self) -> bool {
        self.current_frame_number() == 15
            || self.current_frame_number() == 30
            || self.current_frame_number() == 45
            || self.current_frame_number() == 60
    }
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct RevealAnimation(ReversableBasicAnimation);

impl RevealAnimation {
    fn new() -> Self {
        Self(ReversableBasicAnimation::new(100, false))
    }
    fn get_opacity(&self) -> f32 {
        self.progress(Easing::OutElastic)
    }
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct PlayShardAnimation(BasicAnimation);

impl PlayShardAnimation {
    fn new() -> Self {
        Self(BasicAnimation::new(100, false))
    }
    fn get_opacity(&self) -> f32 {
        1.0 - self.progress(Easing::OutCubic)
    }
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct ShardRotationAnimation(CircularAnimation);

impl ShardRotationAnimation {
    fn new() -> Self {
        Self(CircularAnimation::new(400, false))
    }
    /// Scaled to PI not to a 100%.
    fn get_rotation(&self) -> f32 {
        self.progress(Easing::Linear) * 2.0 * PI
    }
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct InterpolationAnimation(BasicAnimation);

impl InterpolationAnimation {
    fn new() -> Self {
        Self(BasicAnimation::new(100, false))
    }
    fn get_progress(&self) -> f32 {
        self.progress(Easing::OutElastic)
    }
}

#[derive(Clone, Debug)]
pub struct ViewableAvatar {
    window_size: Size,
    id: PlayerId,
    name: String,
    my_turn: bool,
    avatar: Avatar,
    shards: usize,
    interpolation: bool,
    sprite_animation: SpriteAnimation,
    reveal_animation: RevealAnimation,
    play_shard_animation: PlayShardAnimation,
    shard_rotation_animation: ShardRotationAnimation,
    interpolation_animation: InterpolationAnimation,
}

impl ViewableAvatar {
    pub fn new(window_size: Size, avatar_kind: AvatarKind, id: PlayerId, name: String) -> Self {
        let mut viewable_avatar = Self {
            window_size,
            id,
            name,
            my_turn: false,
            avatar: avatar_kind.to_avatar(),
            shards: 20,
            interpolation: false,
            sprite_animation: SpriteAnimation::new(),
            reveal_animation: RevealAnimation::new(),
            play_shard_animation: PlayShardAnimation::new(),
            shard_rotation_animation: ShardRotationAnimation::new(),
            interpolation_animation: InterpolationAnimation::new(),
        };
        viewable_avatar
            .interpolation_animation
            .on_end_reached(AvatarMessage::InterpolationEnded(id).convert_msg());
        viewable_avatar.sprite_animation.start_infinite();
        viewable_avatar.shard_rotation_animation.start_infinite();
        viewable_avatar
    }
    pub fn id(&self) -> PlayerId {
        self.id
    }
    fn avatar_img_position(&self) -> Point {
        let size: f32 = self.window_size.width * AVATAR_SHARD_SIZE_MULT_WITH_WINDOW_WIDTH;
        Point::new(size, size)
    }
    fn shard_position(&self, shard_number: i64) -> Point {
        if self.interpolation {
            self.interpolated_position(shard_number)
        } else {
            let shard_size: f32 = self.window_size.width * AVATAR_SHARD_SIZE_MULT_WITH_WINDOW_WIDTH;
            let circle_radius: f32 = self.width() / 2.0;
            let mut x: f32 = circle_radius - shard_size / 2.0;
            let mut y: f32 = x;
            let rotation: f32 = self.shard_position_rotation_angle(shard_number, 0);
            x += x * rotation.cos();
            y += y * rotation.sin();
            Point::new(x, y)
        }
    }
    fn shard_position_rotation_angle(&self, shard_number: i64, adjust: i64) -> f32 {
        // The angle is scaled from 0.0 to PI, not from 0.0 to 1.0.
        (((shard_number as i64 + adjust) as f32 / (self.shards as i64 + adjust) as f32) * 2.0 * PI
            + self.shard_rotation_animation.get_rotation())
            - (PI / 2.0) // To start on top of the circle.
    }
    fn interpolated_position(&self, shard_number: i64) -> Point {
        let shard_size: f32 = self.window_size.width * AVATAR_SHARD_SIZE_MULT_WITH_WINDOW_WIDTH;
        let circle_radius: f32 = self.width() / 2.0;
        let mut x: f32 = circle_radius - shard_size / 2.0;
        let mut y: f32 = x;
        let rotation_before: f32 = self.shard_position_rotation_angle(shard_number, 0);
        let rotation_after: f32 = self.shard_position_rotation_angle(shard_number, -1);
        let rotation: f32 = rotation_before
            + (rotation_after - rotation_before) * self.interpolation_animation.get_progress();
        x += x * rotation.cos();
        y += y * rotation.sin();
        Point::new(x, y)
    }
    fn shard_rotation(&self, shard_number: i64) -> f32 {
        let rotation_before: f32 = self.shard_rotation_helper(shard_number, 0);
        if self.interpolation {
            let rotation_after: f32 = self.shard_rotation_helper(shard_number, -1);
            rotation_before
                + (rotation_after - rotation_before) * self.interpolation_animation.get_progress()
        } else {
            rotation_before
        }
    }
    fn shard_rotation_helper(&self, shard_number: i64, adjust: i64) -> f32 {
        ((shard_number as i64 + adjust) as f32 / (self.shards as i64 + adjust) as f32) * 2.0 * PI
            + self.shard_rotation_animation.get_rotation()
    }
    fn sprite_size(&self) -> f32 {
        AVATAR_IMG_SIZE_MULT_WITH_WINDOW_WIDTH * self.window_size.width
    }
    fn sprite<'a>(&self, pose: AvatarPose, compare_pose: AvatarPose) -> Pin<'a, AppMessage> {
        let sprite_size: f32 = self.sprite_size();
        let opacity: f32 = if pose == compare_pose { 1.0 } else { 0.0 };
        pin(
            iced::widget::image(self.avatar.kind().img_path(compare_pose))
                // AI-Usage: Claude for learning filter_method to achieve non blurred pixel art.
                .filter_method(FilterMethod::Nearest)
                .opacity(opacity)
                .width(sprite_size)
                .height(sprite_size),
        )
        .position(self.avatar_img_position())
    }
    fn text_size(&self) -> f32 {
        self.width() / 8.0
    }
}

impl Notifiable for ViewableAvatar {
    type OwnMessage = AvatarMessage;
    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage> {
        match msg {
            AvatarMessage::AddShards(id, shards) => {
                if self.id == id {
                    self.shards = shards;
                    return self.reveal_animation.start_force();
                }
            }
            AvatarMessage::PlayShard(id) => {
                if self.id == id && self.shards > 0 {
                    let mut tb = TaskBatcher::new();
                    self.avatar.start_casting();
                    self.interpolation = true;
                    tb.push(self.play_shard_animation.start_force());
                    tb.push(self.sprite_animation.start_force());
                    tb.push(self.interpolation_animation.start_force());
                    return tb.batch();
                }
            }
            AvatarMessage::InterpolationEnded(id) => {
                if self.id == id && self.shards > 0 {
                    self.shards -= 1;
                    self.interpolation = false;
                    return self.play_shard_animation.reset();
                }
            }
            AvatarMessage::ChangeTurn(id) => {
                if id == self.id {
                    self.my_turn = true;
                } else {
                    self.my_turn = false;
                }
            }
            AvatarMessage::NobodiesTurn => {
                self.my_turn = false;
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
        tb.push(self.play_shard_animation.next_frame());
        tb.push(self.shard_rotation_animation.next_frame());
        tb.push(self.interpolation_animation.next_frame());
        tb.batch()
    }
}

impl Resizable for ViewableAvatar {
    fn update_size(&mut self, window_size: Size) {
        self.window_size = window_size;
    }
    fn width(&self) -> f32 {
        self.window_size.width * AVATAR_SIZE_MULT_WITH_WINDOW_WIDTH
    }
    fn height(&self) -> f32 {
        self.width() * AVATAR_FRAME_WIDTH_HEIGHT_RATIO
    }
}

impl SizeFromOutside for ViewableAvatar {
    fn width_for(window_size: Size) -> f32 {
        window_size.width * AVATAR_SIZE_MULT_WITH_WINDOW_WIDTH
    }
    fn height_for(window_size: Size) -> f32 {
        Self::width_for(window_size) * AVATAR_FRAME_WIDTH_HEIGHT_RATIO
    }
}

impl Viewable for ViewableAvatar {
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let mut avatar = stack!().width(self.width()).height(self.height());
        if self.my_turn {
            avatar = avatar.push(
                image("assets/avatars/avatar_frame_turn.png").filter_method(FilterMethod::Nearest),
            );
        } else {
            avatar = avatar.push(
                image("assets/avatars/avatar_frame_idle.png").filter_method(FilterMethod::Nearest),
            );
        }
        avatar = avatar.push(self.sprite(self.avatar.pose(), AvatarPose::Casting1));
        avatar = avatar.push(self.sprite(self.avatar.pose(), AvatarPose::Casting2));
        avatar = avatar.push(self.sprite(self.avatar.pose(), AvatarPose::Standing1));
        avatar = avatar.push(self.sprite(self.avatar.pose(), AvatarPose::Standing2));
        if self.shards > 0 {
            let play_opacity: f32 = self.play_shard_animation.get_opacity();
            let shard_size: f32 = self.window_size.width * AVATAR_SHARD_SIZE_MULT_WITH_WINDOW_WIDTH;
            for shard in 0..self.shards {
                let opacity: f32 = if shard == self.shards - 1 {
                    self.reveal_animation.get_opacity().min(play_opacity)
                } else {
                    self.reveal_animation.get_opacity()
                };
                avatar = avatar.push(
                    pin(image(self.avatar.kind().shard_path())
                        .rotation(self.shard_rotation(shard as i64))
                        .scale(0.8)
                        .opacity(opacity)
                        .width(shard_size)
                        .height(shard_size))
                    .position(self.shard_position(shard as i64)),
                );
            }
        }
        avatar = avatar.push(
            pin(container(
                text(self.name.clone())
                    .size(self.text_size())
                    .color(Color::from_rgb(1.0, 0.85, 0.4)),
            )
            .width(self.width())
            .height(self.height() - self.sprite_size() - self.avatar_img_position().y)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center))
            .position(Point::new(
                0.0,
                // 1.7 is an arbitrary number that results into a good text position.
                self.sprite_size() + self.avatar_img_position().y * 1.7,
            )),
        );
        Container::new(avatar)
            .width(self.width())
            .height(self.height())
    }
}
