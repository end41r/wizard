pub mod avatar;
pub mod middle;

use crate::{
    api::{AvatarKind, Player, PlayerId},
    client::{AppMessage, TaskBatcher},
    gameplay_ui::{
        hand::HandMessage,
        table::{
            avatar::{AvatarMessage, ViewableAvatar},
            middle::{TableMiddleMessage, ViewableTableMiddle},
        },
        GameViewMessage,
    },
    ui_element_traits::*,
};
use iced::{
    widget::{container, stack, Container},
    Size, Task,
};

#[derive(Debug, Clone)]
pub enum TableMessage {
    TableMiddleMessage(TableMiddleMessage),
    AvatarMessage(AvatarMessage),
    BuildAvatars(PlayerId, Vec<Player>),
}

impl Message for TableMessage {
    fn convert_msg_from(msg: Self) -> AppMessage {
        GameViewMessage::convert_msg_from(GameViewMessage::TableMessage(msg))
    }
}

pub struct ViewableTable {
    window_size: Size,
    middle: ViewableTableMiddle,
    my_avatar: ViewableAvatar,
    other_avatars: Vec<ViewableAvatar>,
}

impl ViewableTable {
    pub fn new(window_size: Size) -> Self {
        Self {
            window_size,
            middle: ViewableTableMiddle::new(window_size),
            my_avatar: ViewableAvatar::new(window_size, AvatarKind::Mage),
            other_avatars: Self::build_test_avatars(window_size),
        }
    }

    pub fn build_test_avatars(window_size: Size) -> Vec<ViewableAvatar> {
        vec![
            ViewableAvatar::new(window_size, AvatarKind::Elf),
            ViewableAvatar::new(window_size, AvatarKind::Knight),
            ViewableAvatar::new(window_size, AvatarKind::Mage),
            ViewableAvatar::new(window_size, AvatarKind::Witch),
            ViewableAvatar::new(window_size, AvatarKind::Elf),
        ]
    }
}

impl Notifiable for ViewableTable {
    type OwnMessage = TableMessage;

    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage> {
        match msg {
            TableMessage::TableMiddleMessage(table_middle_msg) => {
                self.middle.update_with_msg(table_middle_msg)
            }
            TableMessage::AvatarMessage(avatar_msg) => {
                let mut tb = TaskBatcher::new();
                for avatar in self.other_avatars.iter_mut() {
                    tb.push(avatar.update_with_msg(avatar_msg.clone()));
                }
                tb.push(self.my_avatar.update_with_msg(avatar_msg));
                tb.batch()
            }
            TableMessage::BuildAvatars(my_id, players) => {
                for player in players.iter() {
                    if player.id == my_id {
                        self.my_avatar = ViewableAvatar::new(self.window_size, player.avatar)
                    } else {
                        self.other_avatars
                            .push(ViewableAvatar::new(self.window_size, player.avatar));
                    }
                }
                Task::none()
            }
        }
    }
}

impl Animated for ViewableTable {
    fn update_animations(&mut self) -> Task<AppMessage> {
        let mut tb = TaskBatcher::new();
        tb.push(self.middle.update_animations());
        for avatar in self.other_avatars.iter_mut() {
            tb.push(avatar.update_animations());
        }
        tb.push(self.my_avatar.update_animations());
        tb.batch()
    }
}

impl Resizable for ViewableTable {
    fn height(&self) -> f32 {
        self.middle.height() * 2.0
    }
    fn width(&self) -> f32 {
        self.middle.width() * 2.0
    }
    fn update_size(&mut self, window_size: iced::Size) {
        self.window_size = window_size;
        self.middle.update_size(window_size);
        for avatar in self.other_avatars.iter_mut() {
            avatar.update_size(window_size);
        }
        self.my_avatar.update_size(window_size);
    }
}

impl Viewable for ViewableTable {
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let mut content = stack!().width(self.width()).height(self.height());
        // Table Middle
        content = content.push(
            self.middle
                .view_and_move(self.middle.width() * 0.5, self.middle.height() * 0.15),
        );
        // Player Avatars
        let avatar_size: f32 = ViewableAvatar::width_for(self.window_size);
        let sec_col_x_spawn: f32 = self.width() - avatar_size;
        content = content.push(self.my_avatar.view());
        content = content.push(self.other_avatars[0].view_and_move(sec_col_x_spawn, 0.0));
        content = content.push(self.other_avatars[1].view_and_move(0.0, avatar_size * 1.5));
        if self.other_avatars.len() > 2 {
            content = content
                .push(self.other_avatars[2].view_and_move(sec_col_x_spawn, avatar_size * 1.5))
        }
        if self.other_avatars.len() > 3 {
            content = content.push(self.other_avatars[3].view_and_move(0.0, avatar_size * 3.0))
        }
        if self.other_avatars.len() > 4 {
            content = content
                .push(self.other_avatars[4].view_and_move(sec_col_x_spawn, avatar_size * 3.0))
        }
        container(content)
    }
}
