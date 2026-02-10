pub mod middle;

use crate::gameplay_ui::hand::ViewableHand;
use crate::{
    api::Card,
    client::AppMessage,
    gameplay_ui::{
        hand::HandMessage,
        table::middle::{card_deck::CardDeckMessage, TableMiddleMessage, ViewableTableMiddle},
    },
    ui_element_traits::*,
};
use iced::{widget::Container, Size, Task};

#[derive(Debug, Clone)]
pub enum TableMessage {
    TableMiddleMessage(TableMiddleMessage),
}

impl TableMessage {
    pub fn notify_hand(&self, viewable_table: &ViewableTable) -> Task<AppMessage> {
        let cards_dealt: Option<(usize, Option<Card>)> = match self {
            TableMessage::TableMiddleMessage(table_middle_msg) => match table_middle_msg {
                TableMiddleMessage::CardDeckMessage(card_deck_msg) => match card_deck_msg {
                    CardDeckMessage::Deal(amount, trump) => Some((*amount, *trump)),
                    _ => None,
                },
                _ => None,
            },
        };
        if cards_dealt.is_some() {
            return Task::done(ViewableHand::convert_msg(HandMessage::DrawCards(
                ViewableHand::build_test_cards(viewable_table.window_size),
            )));
        };
        Task::none()
    }
}

pub struct ViewableTable {
    window_size: Size,
    middle: ViewableTableMiddle,
}

impl ViewableTable {
    pub fn new(window_size: Size) -> Self {
        Self {
            window_size,
            middle: ViewableTableMiddle::new(window_size),
        }
    }
}

impl Message for ViewableTable {
    type OwnMessage = TableMessage;
    fn convert_msg(msg: Self::OwnMessage) -> AppMessage {
        AppMessage::TableMessage(msg)
    }
    fn update_with_msg(&mut self, msg: Self::OwnMessage) {
        match msg {
            TableMessage::TableMiddleMessage(table_middle_msg) => {
                self.middle.update_with_msg(table_middle_msg)
            }
        }
    }
}

impl Animated for ViewableTable {
    fn update_animations(&mut self) {
        self.middle.update_animations();
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
