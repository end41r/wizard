pub mod deck_card;
pub mod trump_card;

use crate::{
    animation::AnimationStarter,
    api::{Card, CARD_BACK_PATH},
    client::{AppMessage, TaskBatcher},
    gameplay_ui::{
        card_area_middle_space_heigth, card_area_middle_space_width, card_area_middle_spawn_point,
        card_img_middle_base_scale,
        hand::ViewableHand,
        table::{
            middle::{
                card_deck::{
                    deck_card::ViewableDeckCard,
                    trump_card::{TrumpCardMessage, ViewableTrumpCard},
                },
                TableMiddleMessage,
            },
            HandMessage,
        },
    },
    ui_element_traits::*,
};
use iced::{
    widget::{image, pin, stack, Container},
    Size, Task,
};

type TrumpCard = Card;

#[derive(Debug, Clone)]
pub enum CardDeckMessage {
    AllDealt,
    Deal(usize, Option<TrumpCard>),
    Shuffle,
    ClearDeckCards,
    AddDeckCard(usize),
    TrumpCardMessage(TrumpCardMessage),
}

impl Message for CardDeckMessage {
    fn convert_msg_from(msg: Self) -> AppMessage {
        TableMiddleMessage::convert_msg_from(TableMiddleMessage::CardDeckMessage(msg))
    }
}

impl ReplaceUsize for CardDeckMessage {
    fn replace_usize(&self, value: usize) -> Self {
        match self {
            CardDeckMessage::AddDeckCard(_) => CardDeckMessage::AddDeckCard(value),
            CardDeckMessage::Deal(_, trump_card) => CardDeckMessage::Deal(value, *trump_card),
            CardDeckMessage::AllDealt => self.clone(),
            CardDeckMessage::TrumpCardMessage(_) => self.clone(),
            CardDeckMessage::ClearDeckCards => self.clone(),
            CardDeckMessage::Shuffle => self.clone(),
        }
    }
}

pub struct ViewableCardDeck {
    window_size: Size,
    show_base_card: bool,
    trump_card: Option<ViewableTrumpCard>,
    deck_cards: Vec<ViewableDeckCard>,
    deal_card_animation_starter: AnimationStarter<CardDeckMessage, CardDeckMessage>,
}

impl ViewableCardDeck {
    pub fn new(window_size: Size) -> Self {
        let mut viewable_card_deck = Self {
            window_size,
            show_base_card: true,
            trump_card: None,
            deck_cards: Vec::new(),
            deal_card_animation_starter: AnimationStarter::new(
                3,
                5,
                CardDeckMessage::AddDeckCard(0),
            ),
        };
        viewable_card_deck
            .deal_card_animation_starter
            .on_all_ended(CardDeckMessage::AllDealt);
        viewable_card_deck
    }
}

impl Notifiable for ViewableCardDeck {
    type OwnMessage = CardDeckMessage;

    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage> {
        match msg {
            CardDeckMessage::AddDeckCard(cycle) => {
                let view_able_deck_card = ViewableDeckCard::new(self.window_size, cycle);
                self.deck_cards.push(view_able_deck_card);
            }
            CardDeckMessage::ClearDeckCards => {
                self.deck_cards.clear();
            }
            CardDeckMessage::Deal(cards, trump_card) => {
                self.deal_card_animation_starter.start(cards);
                if trump_card.is_some() {
                    self.trump_card = Some(ViewableTrumpCard::new(
                        self.window_size,
                        trump_card.unwrap(),
                    ));
                } else {
                    self.trump_card = None;
                }
                return HandMessage::DrawCards(ViewableHand::build_test_cards(self.window_size))
                    .convert_msg_to_task();
            }
            CardDeckMessage::AllDealt => {
                return TaskBatcher::instant_batch([
                    CardDeckMessage::ClearDeckCards.convert_msg_to_task(),
                    TrumpCardMessage::TurnPart1.convert_msg_to_task(),
                ])
            }
            CardDeckMessage::TrumpCardMessage(trump_card_msg) => {
                if self.trump_card.is_some() {
                    return self
                        .trump_card
                        .as_mut()
                        .unwrap()
                        .update_with_msg(trump_card_msg);
                }
            }
            CardDeckMessage::Shuffle => {}
        }
        Task::none()
    }
}

impl Animated for ViewableCardDeck {
    fn update_animations(&mut self) -> Task<AppMessage> {
        let mut tb = TaskBatcher::new();
        tb.push(self.deal_card_animation_starter.next_frame());
        if self.trump_card.is_some() {
            tb.push(self.trump_card.as_mut().unwrap().update_animations());
        }
        for card in self.deck_cards.iter_mut() {
            tb.push(card.update_animations());
        }
        tb.batch()
    }
}

impl Resizable for ViewableCardDeck {
    fn height(&self) -> f32 {
        card_area_middle_space_heigth(self.window_size)
    }
    fn width(&self) -> f32 {
        card_area_middle_space_width(self.window_size)
    }
    fn update_size(&mut self, window_size: Size) {
        self.window_size = window_size;
        for card in self.deck_cards.iter_mut() {
            card.update_size(window_size);
        }
        if self.trump_card.is_some() {
            self.trump_card.as_mut().unwrap().update_size(window_size);
        }
    }
}

impl Viewable for ViewableCardDeck {
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let mut card_stack = stack!();
        if self.show_base_card {
            let width = ViewableDeckCard::width_for(self.window_size);
            let heigth = ViewableDeckCard::height_for(self.window_size);
            let spawn_point = card_area_middle_spawn_point(width, heigth, self.window_size);
            let img = pin(image(CARD_BACK_PATH.to_string())
                .width(width)
                .height(heigth)
                .scale(card_img_middle_base_scale()))
            .position(spawn_point);
            card_stack = card_stack.push(img);
        }
        if self.trump_card.is_some() {
            let trump_card = self.trump_card.as_ref().unwrap();
            let spawn_point = card_area_middle_spawn_point(
                trump_card.width(),
                trump_card.height(),
                self.window_size,
            );
            card_stack = card_stack.push(trump_card.view_and_move(spawn_point.x, spawn_point.y));
        };
        for card in self.deck_cards.iter() {
            let spawn_point =
                card_area_middle_spawn_point(card.width(), card.height(), self.window_size);
            card_stack = card_stack.push(card.view_and_move(
                spawn_point.x + card.offset().x,
                spawn_point.y + card.offset().y,
            ));
        }
        Container::new(card_stack)
            .width(self.width())
            .height(self.height())
    }
}
