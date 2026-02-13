pub mod stack_card;
pub mod glow_card;

use crate::{
    animation::AnimationStarter, api::Card, client::{AppMessage, TaskBatcher}, gameplay_ui::{
        card_area_middle_space_heigth, card_area_middle_space_width, card_area_middle_spawn_point,
        table::middle::{TableMiddleMessage, card_deck::CardDeckMessage, card_stack::stack_card::ViewableStackCard},
    }, ui_element_traits::*
};
use iced::{
    widget::{Container, Stack},
    Size, Task,
};

#[derive(Debug, Clone)]
pub enum CardStackMessage {
    CardPlayed(Card),
    HideAllCard,
    HideCard(usize),
    RemoveAllCards,
}

impl Message for CardStackMessage {
    fn convert_msg_from(msg: Self) -> AppMessage {
        TableMiddleMessage::convert_msg_from(TableMiddleMessage::CardStackMessage(msg))
    }
}

impl ReplaceUsize for CardStackMessage {
    fn replace_usize(&self, value: usize) -> Self {
        match self {
            CardStackMessage::HideCard(_) => CardStackMessage::HideCard(value),
            CardStackMessage::HideAllCard => self.clone(),
            CardStackMessage::CardPlayed(_) => self.clone(),
            CardStackMessage::RemoveAllCards => self.clone(),
        }
    }
}

pub struct ViewableCardStack {
    window_size: Size,
    cards: Vec<ViewableStackCard>,
    clear_card_stack_animation_starter: AnimationStarter<CardStackMessage, CardStackMessage>
}

impl ViewableCardStack {
    pub fn new(window_size: Size) -> Self {
        let mut viewable_stack_card = Self {
            window_size,

            cards: Vec::new(),
            clear_card_stack_animation_starter: AnimationStarter::new(10, 20, CardStackMessage::HideCard(0))
        };
        viewable_stack_card.clear_card_stack_animation_starter.on_all_ended(CardStackMessage::RemoveAllCards);
        viewable_stack_card
    }
}

impl Notifiable for ViewableCardStack {
    type OwnMessage = CardStackMessage;

    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage> {
        match msg {
            CardStackMessage::CardPlayed(card) => {
                self.cards.push(ViewableStackCard::new(self.window_size, card));
                if self.cards.len() == 1 {
                    return CardDeckMessage::ChangeGlow(card).convert_msg_to_task();
                }
            },
            CardStackMessage::HideAllCard => {
                self.clear_card_stack_animation_starter.start(self.cards.len() - 1);
            },
            CardStackMessage::HideCard(id) => {
                let card_count: usize = self.cards.len();
                self.cards[card_count - 1 - id].remove_animation.start();
            }
            CardStackMessage::RemoveAllCards => {
                self.cards.clear();
            }
        };
        Task::none()
    }
}

impl Animated for ViewableCardStack {
    fn update_animations(&mut self) -> Task<AppMessage> {
        let mut tb = TaskBatcher::new();
        tb.push(self.clear_card_stack_animation_starter.next_frame());
        for card in self.cards.iter_mut() {
            tb.push(card.update_animations());
        }
        tb.batch()
    }
}

impl Resizable for ViewableCardStack {
    fn height(&self) -> f32 {
        card_area_middle_space_heigth(self.window_size)
    }
    fn width(&self) -> f32 {
        card_area_middle_space_width(self.window_size)
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
            let spawn_point =
                card_area_middle_spawn_point(card.width(), card.height(), self.window_size);
            card_stack = card_stack.push(card.view_and_move(spawn_point.x, spawn_point.y))
        }
        Container::new(card_stack)
            .width(self.width())
            .height(self.height())
    }
}
