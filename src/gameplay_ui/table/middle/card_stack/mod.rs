pub mod stack_card;

use std::ops::Not;

use crate::{
    animation::{AnimationStarter, Easing, ReversableBasicAnimation},
    api::Card,
    client::{AppMessage, TaskBatcher},
    gameplay_ui::{
        card_area_middle_space_height, card_area_middle_space_width, card_area_middle_spawn_point,
        table::middle::{
            card_deck::CardDeckMessage, card_stack::stack_card::ViewableStackCard,
            TableMiddleMessage,
        },
        CARD_WIDTH_HEIGHT_RATIO,
    },
    ui_element_traits::*,
};
use derive_more::{Deref, DerefMut};
use iced::{
    mouse::Interaction,
    widget::{container, image, pin, Container, MouseArea, Stack},
    Point, Size, Task,
};

#[derive(Debug, Clone)]
pub enum CardStackMessage {
    CardPlayed(Card),
    HideAllCards,
    HideCard(usize),
    RemoveAllCards,
    ShowPlayedCards,
    HidePlayedCards,
    SwitchAlwaysShowPlayedCards,
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
            CardStackMessage::HideAllCards => self.clone(),
            CardStackMessage::CardPlayed(_) => self.clone(),
            CardStackMessage::RemoveAllCards => self.clone(),
            CardStackMessage::ShowPlayedCards => self.clone(),
            CardStackMessage::HidePlayedCards => self.clone(),
            CardStackMessage::SwitchAlwaysShowPlayedCards => self.clone(),
        }
    }
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct ViewPlayedCardsAnimation(ReversableBasicAnimation);

impl ViewPlayedCardsAnimation {
    pub fn new(duration: usize) -> Self {
        Self(ReversableBasicAnimation::new(duration, false))
    }
    pub fn get_progress(&self) -> f32 {
        self.progress(Easing::OutCubic)
    }
    pub fn get_opacity(&self) -> f32 {
        self.progress(Easing::InOutCubic)
    }
}

pub struct ViewableCardStack {
    window_size: Size,
    cards: Vec<ViewableStackCard>,
    always_show_played_cards: bool,
    view_played_cards_animation: ViewPlayedCardsAnimation,
    clear_card_stack_animation_starter: AnimationStarter<CardStackMessage, CardStackMessage>,
}

impl ViewableCardStack {
    pub fn new(window_size: Size) -> Self {
        let mut viewable_stack_card = Self {
            window_size,
            cards: Vec::new(),
            always_show_played_cards: false,
            view_played_cards_animation: ViewPlayedCardsAnimation::new(40),
            clear_card_stack_animation_starter: AnimationStarter::new(
                10,
                20,
                CardStackMessage::HideCard(0),
            ),
        };
        viewable_stack_card
            .clear_card_stack_animation_starter
            .on_all_ended(CardStackMessage::RemoveAllCards);
        viewable_stack_card
    }
}

impl Notifiable for ViewableCardStack {
    type OwnMessage = CardStackMessage;

    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage> {
        match msg {
            CardStackMessage::CardPlayed(card) => {
                let mut tb = TaskBatcher::new();
                let mut stack_card = ViewableStackCard::new(self.window_size, card);
                tb.push(stack_card.reveal_animation.start());
                self.cards.push(stack_card);
                if self.cards.len() == 1 {
                    tb.push_msg(CardDeckMessage::ChangeGlow(card));
                    tb.push_msg(CardDeckMessage::ShowGlow);
                }
                return tb.batch();
            }
            CardStackMessage::HideAllCards => {
                if self.cards.len() > 0 {
                    return self
                        .clear_card_stack_animation_starter
                        .start(self.cards.len().max(1) - 1);
                }
            }
            CardStackMessage::HideCard(id) => {
                let card_count: usize = self.cards.len();
                return self.cards[card_count - 1 - id].remove_animation.start();
            }
            CardStackMessage::RemoveAllCards => {
                self.cards.clear();
            }
            CardStackMessage::ShowPlayedCards => {
                return self.view_played_cards_animation.start();
            }
            CardStackMessage::HidePlayedCards => {
                if !self.always_show_played_cards {
                    return self.view_played_cards_animation.reverse();
                }
            }
            CardStackMessage::SwitchAlwaysShowPlayedCards => {
                self.always_show_played_cards = self.always_show_played_cards.not();
                if self.always_show_played_cards {
                    return self.view_played_cards_animation.start();
                } else {
                    return self.view_played_cards_animation.reverse();
                }
            }
        };
        Task::none()
    }
}

impl Animated for ViewableCardStack {
    fn update_animations(&mut self) -> Task<AppMessage> {
        let mut tb = TaskBatcher::new();
        tb.push(self.view_played_cards_animation.next_frame());
        tb.push(self.clear_card_stack_animation_starter.next_frame());
        for card in self.cards.iter_mut() {
            tb.push(card.update_animations());
        }
        tb.batch()
    }
}

impl Resizable for ViewableCardStack {
    fn height(&self) -> f32 {
        card_area_middle_space_height(self.window_size)
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
        let mut content = Stack::new().width(self.width()).height(self.height());

        let mut card_stack = Stack::new();
        let stack_card_width: f32 = ViewableStackCard::width_for(self.window_size);
        let stack_card_height: f32 = ViewableStackCard::height_for(self.window_size);
        let spawn_point: Point = card_area_middle_spawn_point(
            ViewableStackCard::width_for(self.window_size),
            ViewableStackCard::height_for(self.window_size),
            self.window_size,
        );
        for card in self.cards.iter() {
            card_stack = card_stack.push(card.view_and_move(spawn_point.x, spawn_point.y))
        }
        card_stack = card_stack.push(
            pin(MouseArea::new(
                container(None::<&str>)
                    .width(stack_card_width)
                    .height(stack_card_height),
            )
            .interaction(Interaction::Pointer)
            .on_enter(CardStackMessage::ShowPlayedCards.convert_msg())
            .on_exit(CardStackMessage::HidePlayedCards.convert_msg())
            .on_press(CardStackMessage::SwitchAlwaysShowPlayedCards.convert_msg()))
            .position(spawn_point),
        );
        content = content.push(pin(card_stack).position(Point::new(0.0, 0.0)));

        if self.cards.len() > 0 {
            let mut cards = Stack::new();
            let card_width: f32 = self.width() / 6.0; // There can be 6 cards played at max.
            let card_height: f32 = card_width * CARD_WIDTH_HEIGHT_RATIO;
            let start_position_x: f32 = (self.width() - card_width) / 2.0;
            let start_position_y: f32 = self.height() - card_height;
            let start_point: Point = Point::new(start_position_x, start_position_y);
            for card_number in 0..self.cards.len() {
                let end_point: Point = Point::new(card_number as f32 * card_width, 0.0);
                let spawn_point: Point = Point::new(
                    start_point.x
                        + (end_point.x - start_point.x)
                            * self.view_played_cards_animation.get_progress(),
                    start_point.y
                        + (end_point.y - start_point.y)
                            * self.view_played_cards_animation.get_progress(),
                );
                cards = cards.push(
                    pin(image(self.cards[card_number].card().img_path())
                        .opacity(self.view_played_cards_animation.get_opacity())
                        .width(card_width)
                        .height(card_height))
                    .position(spawn_point),
                );
            }
            content = content.push(cards);
        };

        Container::new(content)
            .width(self.width())
            .height(self.height())
    }
}
