pub mod stack_card;

use crate::{
    api::Card,
    client::AppMessage,
    gameplay_ui::table::middle::{
        card_stack::stack_card::ViewableStackCard, TableMiddleMessage, ViewableTableMiddle,
    },
    ui_element_traits::*,
};
use iced::{
    widget::{Container, Stack},
    Size,
};

#[derive(Debug, Clone)]
pub enum CardStackMessage {
    CardPlayed(Card),
}

pub struct ViewableCardStack {
    window_size: Size,
    cards: Vec<ViewableStackCard>,
}

impl ViewableCardStack {
    pub fn new(window_size: Size) -> Self {
        Self {
            window_size,
            cards: Vec::new(),
        }
    }
}

impl Message for ViewableCardStack {
    type OwnMessage = CardStackMessage;
    fn convert_msg(msg: Self::OwnMessage) -> AppMessage {
        ViewableTableMiddle::convert_msg(TableMiddleMessage::CardStackMessage(msg))
    }
    fn update_with_msg(&mut self, msg: Self::OwnMessage) {
        match msg {
            CardStackMessage::CardPlayed(card) => self
                .cards
                .push(ViewableStackCard::new(self.window_size, card)),
        }
    }
}

impl Animated for ViewableCardStack {
    fn update_animations(&mut self) {
        for card in self.cards.iter_mut() {
            card.update_animations();
        }
    }
}

impl Resizable for ViewableCardStack {
    fn height(&self) -> f32 {
        ViewableStackCard::height_for(self.window_size)
    }
    fn width(&self) -> f32 {
        ViewableStackCard::width_for(self.window_size)
    }
    fn update_size(&mut self, window_size: Size) {
        self.window_size = window_size;
        for card in self.cards.iter_mut() {
            card.update_size(window_size);
        }
    }
}

impl Viewable for ViewableCardStack {
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let mut card_stack = Stack::new();
        for card in self.cards.iter() {
            card_stack = card_stack.push(card.view())
        }
        Container::new(card_stack)
            .width(self.width())
            .height(self.height())
    }
}
