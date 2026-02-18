pub mod avatar;
pub mod middle;

use std::collections::HashMap;

use crate::{
    api::PlayerId,
    client::{App, AppMessage, TaskBatcher},
    gameplay_ui::{
        hand::HandMessage,
        table::{
            avatar::{Avatar, AvatarMessage},
            middle::{TableMiddleMessage, ViewableTableMiddle},
        },
        GameViewMessage,
    },
    ui_element_traits::*,
};
use iced::{widget::Container, Size, Task};

pub struct AvatarsManager {
    my_id: Option<PlayerId>,
    current_player: Option<PlayerId>,
    pub avatars: HashMap<PlayerId, Avatar>,
}

impl AvatarsManager {
    pub fn build_avatars(&mut self, player_ids: Vec<PlayerId>) {}
    pub fn update(&mut self, app: &App) {
        self.my_id = app.my_id;
        self.current_player = app.current_player;
    }
}

impl Default for AvatarsManager {
    fn default() -> Self {
        Self {
            my_id: None,
            current_player: None,
            avatars: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TableMessage {
    TableMiddleMessage(TableMiddleMessage),
    AvatarMessage(AvatarMessage),
}

impl Message for TableMessage {
    fn convert_msg_from(msg: Self) -> AppMessage {
        GameViewMessage::convert_msg_from(GameViewMessage::TableMessage(msg))
    }
}

pub struct ViewableTable {
    window_size: Size,
    middle: ViewableTableMiddle,
    avatars: AvatarsManager,
}

impl ViewableTable {
    pub fn new(window_size: Size) -> Self {
        Self {
            window_size,
            middle: ViewableTableMiddle::new(window_size),
            avatars: AvatarsManager::default(),
        }
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
                for (_, avatar) in self.avatars.avatars.iter_mut() {
                    tb.push(avatar.update_with_msg(avatar_msg.clone()));
                }
                tb.batch()
            }
        }
    }
}

impl Animated for ViewableTable {
    fn update_animations(&mut self) -> Task<AppMessage> {
        let mut tb = TaskBatcher::new();
        tb.push(self.middle.update_animations());
        for (_, avatar) in self.avatars.avatars.iter_mut() {
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
