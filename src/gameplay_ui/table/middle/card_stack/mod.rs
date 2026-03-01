pub mod stack_card;

use std::ops::Not;

use crate::{
    animation::{BasicAnimation, Easing, ReversableBasicAnimation},
    api::Card,
    client::{audio::Sfx, AppMessage, TaskBatcher},
    gameplay_ui::{
        card_area_middle_space_height, card_area_middle_space_width, card_area_middle_spawn_point,
        table::middle::{
            card_deck::glow_card::GlowMessage, card_stack::stack_card::ViewableStackCard,
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

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct ViewPlayedCardsAnimation(ReversableBasicAnimation);

impl ViewPlayedCardsAnimation {
    pub fn new(duration: usize) -> Self {
        Self(ReversableBasicAnimation::new(duration))
    }
    pub fn get_progress(&self) -> f32 {
        self.progress(Easing::OutCubic)
    }
    pub fn get_opacity(&self) -> f32 {
        self.progress(Easing::InOutCubic)
    }
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct NewCardPlayedAniamtion(BasicAnimation);

impl NewCardPlayedAniamtion {
    pub fn new(duration: usize) -> Self {
        Self(BasicAnimation::new(duration))
    }
    pub fn get_opacity(&self) -> f32 {
        self.progress(Easing::Linear)
    }
}

pub struct ViewableCardStack {
    window_size: Size,
    cards: Vec<ViewableStackCard>,
    always_show_played_cards: bool,
    remove_ready: bool,
    view_played_cards_animation: ViewPlayedCardsAnimation,
    new_card_played_animation: NewCardPlayedAniamtion,
}

impl ViewableCardStack {
    pub fn new(window_size: Size) -> Self {
        Self {
            window_size,
            cards: Vec::new(),
            always_show_played_cards: false,
            remove_ready: false,
            view_played_cards_animation: ViewPlayedCardsAnimation::new(40),
            new_card_played_animation: NewCardPlayedAniamtion::new(20),
        }
    }
}

impl Notifiable for ViewableCardStack {
    type OwnMessage = CardStackMessage;

    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage> {
        match msg {
            CardStackMessage::CardPlayed(card) => {
                let mut tb = TaskBatcher::new();
                let mut stack_card = ViewableStackCard::new(self.window_size, card);
                stack_card.reveal_animation.start();
                self.cards.push(stack_card);
                tb.push_msg(GlowMessage::TryChangeGlow(card));
                tb.push_msg(AppMessage::PlaySfx(Sfx::CardPlay));
                self.new_card_played_animation.start_force();
                if self.cards.len() == 1 && self.always_show_played_cards {
                    self.view_played_cards_animation.start();
                }
                return tb.batch();
            }
            CardStackMessage::HideAllCards => {
                if self.cards.len() > 0 {
                    self.view_played_cards_animation.reverse();
                    for card in self.cards.iter_mut() {
                        card.remove_animation.start();
                    }
                    return GlowMessage::RemoveColor.convert_msg_to_task();
                }
            }
            CardStackMessage::RemoveAllCards => {
                self.remove_ready = true;
            }
            CardStackMessage::ShowPlayedCards => {
                self.view_played_cards_animation.start();
            }
            CardStackMessage::HidePlayedCards => {
                if !self.always_show_played_cards {
                    self.view_played_cards_animation.reverse();
                }
            }
            CardStackMessage::SwitchAlwaysShowPlayedCards => {
                self.always_show_played_cards = self.always_show_played_cards.not();
                if self.always_show_played_cards {
                    self.view_played_cards_animation.start();
                } else {
                    self.view_played_cards_animation.reverse();
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
        tb.push(self.new_card_played_animation.next_frame());
        for card in self.cards.iter_mut() {
            tb.push(card.update_animations());
        }
        if self.remove_ready && self.view_played_cards_animation.current_frame_number() == 0 {
            self.cards.clear();
            self.remove_ready = false;
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
        if self.cards.len() > 0 {
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
        }
        content = content.push(pin(card_stack).position(Point::new(0.0, 0.0)));

        // played cards history
        if !self.cards.is_empty() {
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
                let mut opacity = self.view_played_cards_animation.get_opacity();
                if card_number == self.cards.len() - 1 {
                    // last card
                    opacity = opacity.min(self.new_card_played_animation.get_opacity());
                };
                cards = cards.push(
                    pin(image(self.cards[card_number].card().img_path())
                        .opacity(opacity)
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
