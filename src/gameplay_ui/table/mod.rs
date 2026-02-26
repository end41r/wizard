pub mod avatar;
pub mod middle;

use crate::{
    api::{Player, PlayerId},
    client::{AppMessage, TaskBatcher},
    gameplay_ui::{
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
    DrawShards(usize),
    ChangeTurn(PlayerId),
    NobodiesTurn,
}

impl Message for TableMessage {
    fn convert_msg_from(msg: Self) -> AppMessage {
        GameViewMessage::convert_msg_from(GameViewMessage::TableMessage(msg))
    }
}

pub struct ViewableTable {
    window_size: Size,
    middle: ViewableTableMiddle,
    avatars: Vec<ViewableAvatar>,
}

impl ViewableTable {
    pub fn new(window_size: Size) -> Self {
        Self {
            window_size,
            middle: ViewableTableMiddle::new(window_size),
            avatars: Vec::new(),
        }
    }
    /// This method is highly critical and needs to be executed as soon as possible
    /// which is why this is not handled via a Task.
    pub fn build_avatars(&mut self, players: Vec<Player>) {
        for player in players.iter() {
            self.avatars.push(ViewableAvatar::new(
                self.window_size,
                player.avatar,
                player.id,
            ));
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
                for avatar in self.avatars.iter_mut() {
                    tb.push(avatar.update_with_msg(avatar_msg.clone()));
                }
                tb.batch()
            }
            TableMessage::DrawShards(amount) => {
                let mut tb = TaskBatcher::new();
                for avatar in self.avatars.iter_mut() {
                    tb.push(avatar.update_with_msg(AvatarMessage::AddShards(avatar.id(), amount)))
                }
                tb.batch()
            }
            TableMessage::ChangeTurn(id) => {
                let mut tb = TaskBatcher::new();
                for avatar in self.avatars.iter_mut() {
                    tb.push(avatar.update_with_msg(AvatarMessage::ChangeTurn(id.clone())))
                }
                tb.batch()
            }
            TableMessage::NobodiesTurn => {
                let mut tb = TaskBatcher::new();
                for avatar in self.avatars.iter_mut() {
                    tb.push(avatar.update_with_msg(AvatarMessage::NobodiesTurn))
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
        for avatar in self.avatars.iter_mut() {
            tb.push(avatar.update_animations());
        }
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
        for avatar in self.avatars.iter_mut() {
            avatar.update_size(window_size);
        }
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
        if self.avatars.len() > 0 {
            content = content.push(self.avatars[0].view());
            content = content.push(self.avatars[1].view_and_move(sec_col_x_spawn, 0.0));
            content = content.push(self.avatars[2].view_and_move(0.0, avatar_size * 1.5));
        }
        if self.avatars.len() > 3 {
            content =
                content.push(self.avatars[3].view_and_move(sec_col_x_spawn, avatar_size * 1.5))
        }
        if self.avatars.len() > 4 {
            content = content.push(self.avatars[4].view_and_move(0.0, avatar_size * 3.0))
        }
        if self.avatars.len() > 5 {
            content =
                content.push(self.avatars[5].view_and_move(sec_col_x_spawn, avatar_size * 3.0))
        }
        container(content)
    }
}
