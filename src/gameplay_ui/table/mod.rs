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
use iced::{widget::Container, Size, Task};

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
            other_avatars: Vec::new(),
        }
    }
    pub fn all_avatars(&self) -> Vec<ViewableAvatar> {
        let mut avatars: Vec<ViewableAvatar> = self.other_avatars.clone();
        avatars.push(self.my_avatar.clone());
        avatars
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
                for avatar in self.all_avatars().iter_mut() {
                    tb.push(avatar.update_with_msg(avatar_msg.clone()));
                }
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
                println!("{:?}", self.my_avatar);
                println!("{:?}", self.other_avatars);
                Task::none()
            }
        }
    }
}

impl Animated for ViewableTable {
    fn update_animations(&mut self) -> Task<AppMessage> {
        let mut tb = TaskBatcher::new();
        tb.push(self.middle.update_animations());
        for avatar in self.all_avatars().iter_mut() {
            tb.push(avatar.update_animations());
        }
        tb.batch()
    }
}

impl Resizable for ViewableTable {
    fn height(&self) -> f32 {
        self.middle.height()
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
