pub mod deck_card;
pub mod glow_card;
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
                    glow_card::{CardStackGlow, GlowMessage},
                    trump_card::{TrumpCardMessage, ViewableTrumpCard},
                },
                card_stack::CardStackMessage,
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
    AllCleared,
    Deal(usize, Option<TrumpCard>),
    Shuffle,
    ClearTrumpCard,
    AddDeckCard(usize),
    DealDeckCard(usize),
    TrumpCardMessage(TrumpCardMessage),
    ChangeGlow(Card),
    GlowMessage(GlowMessage),
    ShowGlow,
}

impl Message for CardDeckMessage {
    fn convert_msg_from(msg: Self) -> AppMessage {
        TableMiddleMessage::convert_msg_from(TableMiddleMessage::CardDeckMessage(msg))
    }
}

impl ReplaceUsize for CardDeckMessage {
    fn replace_usize(&self, value: usize) -> Self {
        match self {
            CardDeckMessage::DealDeckCard(_) => CardDeckMessage::DealDeckCard(value),
            CardDeckMessage::AddDeckCard(_) => CardDeckMessage::AddDeckCard(value),
            CardDeckMessage::Deal(_, trump_card) => CardDeckMessage::Deal(value, *trump_card),
            CardDeckMessage::AllDealt => self.clone(),
            CardDeckMessage::AllCleared => self.clone(),
            CardDeckMessage::TrumpCardMessage(_) => self.clone(),
            CardDeckMessage::ClearTrumpCard => self.clone(),
            CardDeckMessage::Shuffle => self.clone(),
            CardDeckMessage::ChangeGlow(_) => self.clone(),
            CardDeckMessage::GlowMessage(_) => self.clone(),
            CardDeckMessage::ShowGlow => self.clone()
        }
    }
}

pub struct ViewableCardDeck {
    window_size: Size,
    glow: CardStackGlow,
    show_base_card: bool,
    deal_msg: Option<CardDeckMessage>,
    trump_card: Option<ViewableTrumpCard>,
    deck_cards: Vec<ViewableDeckCard>,
    deal_card_animation_starter: AnimationStarter<CardDeckMessage, CardDeckMessage>,
    clear_card_animation_starter: AnimationStarter<CardDeckMessage, CardDeckMessage>,
}

impl ViewableCardDeck {
    pub fn new(window_size: Size) -> Self {
        let mut viewable_card_deck = Self {
            window_size,
            glow: CardStackGlow::new(window_size),
            show_base_card: true,
            deal_msg: None,
            trump_card: None,
            deck_cards: Vec::new(),
            deal_card_animation_starter: AnimationStarter::new(
                3,
                10,
                CardDeckMessage::DealDeckCard(0),
            ),
            clear_card_animation_starter: AnimationStarter::new(
                3,
                10,
                CardDeckMessage::AddDeckCard(0),
            ),
        };
        viewable_card_deck
            .deal_card_animation_starter
            .on_all_ended(CardDeckMessage::AllDealt);
        viewable_card_deck
            .clear_card_animation_starter
            .on_all_ended(CardDeckMessage::AllCleared);
        viewable_card_deck
    }
}

impl Notifiable for ViewableCardDeck {
    type OwnMessage = CardDeckMessage;

    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage> {
        match msg {
            CardDeckMessage::AddDeckCard(cycle) => {
                let view_able_deck_card = ViewableDeckCard::new(self.window_size, cycle, true);
                self.deck_cards.push(view_able_deck_card);
            }
            CardDeckMessage::DealDeckCard(cycle) => {
                let view_able_deck_card = ViewableDeckCard::new(self.window_size, cycle, false);
                self.deck_cards.push(view_able_deck_card);
            }
            CardDeckMessage::ClearTrumpCard => {
                self.trump_card = None;
                self.clear_card_animation_starter
                    .start(self.deal_card_animation_starter.times());
            }
            CardDeckMessage::Deal(cards, trump_card) => {
                let mut tb = TaskBatcher::new();
                if self.deal_msg.is_none() {
                    self.deal_msg = Some(CardDeckMessage::Deal(cards, trump_card));
                    self.deal_card_animation_starter.start(cards);
                    if trump_card.is_some() {
                        self.trump_card = Some(ViewableTrumpCard::new(
                            self.window_size,
                            trump_card.unwrap(),
                        ));
                    } else {
                        self.trump_card = None;
                    }
                    tb.push(
                        HandMessage::DrawCards(ViewableHand::build_test_cards(self.window_size))
                            .convert_msg_to_task(),
                    );
                } else {
                    self.deal_msg = Some(CardDeckMessage::Deal(cards, trump_card));
                    tb.push(CardDeckMessage::Shuffle.convert_msg_to_task());
                    tb.push(CardStackMessage::HideAllCard.convert_msg_to_task());
                }
                return tb.batch();
            }
            CardDeckMessage::AllDealt => {
                self.deck_cards.clear();
                return TrumpCardMessage::TurnPart1.convert_msg_to_task();
            }
            CardDeckMessage::AllCleared => {
                if self.deal_msg.is_some() {
                    let deal_msg_copy: Option<CardDeckMessage> = self.deal_msg.clone();
                    self.deal_msg = None;
                    return deal_msg_copy.unwrap().convert_msg_to_task();
                }
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
            CardDeckMessage::Shuffle => {
                self.glow.reveal_animation.reverse();
                return TrumpCardMessage::RemovePart1.convert_msg_to_task();
            }
            CardDeckMessage::ChangeGlow(card) => {
                self.glow.change_color(card);
            }
            CardDeckMessage::GlowMessage(glow_msg) => {
                return self.glow.update_with_msg(glow_msg);
            }
            CardDeckMessage::ShowGlow => {
                self.glow.reveal_animation.start_force();
            }
        }
        Task::none()
    }
}

impl Animated for ViewableCardDeck {
    fn update_animations(&mut self) -> Task<AppMessage> {
        let mut tb = TaskBatcher::new();
        tb.push(self.deal_card_animation_starter.next_frame());
        tb.push(self.clear_card_animation_starter.next_frame());
        if self.trump_card.is_some() {
            tb.push(self.trump_card.as_mut().unwrap().update_animations());
        }
        tb.push(self.glow.update_animations());
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
        self.glow.update_size(window_size);
    }
}

impl Viewable for ViewableCardDeck {
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let mut card_stack = stack!();

        let width: f32 = ViewableDeckCard::width_for(self.window_size);
        let heigth: f32 = ViewableDeckCard::height_for(self.window_size);
        let spawn_point: iced::Point =
            card_area_middle_spawn_point(width, heigth, self.window_size);
        card_stack = card_stack.push(self.glow.view_and_move(spawn_point.x, spawn_point.y));
        if self.show_base_card {
            // Construct base card template
            let base_card = pin(image(CARD_BACK_PATH.to_string())
                .width(width)
                .height(heigth)
                .scale(card_img_middle_base_scale()))
            .position(spawn_point);
            card_stack = card_stack.push(base_card);
        }
        for card in self.deck_cards.iter() {
            let spawn_point =
                card_area_middle_spawn_point(card.width(), card.height(), self.window_size);
            card_stack = card_stack.push(card.view_and_move(
                spawn_point.x + card.offset().x,
                spawn_point.y + card.offset().y,
            ));
        }
        if self.trump_card.is_some() {
            let trump_card = self.trump_card.as_ref().unwrap();
            let spawn_point = card_area_middle_spawn_point(
                trump_card.width(),
                trump_card.height(),
                self.window_size,
            );
            card_stack = card_stack.push(trump_card.view_and_move(spawn_point.x, spawn_point.y));
        } else {
            // Construct base card template
            let base_card = pin(image(CARD_BACK_PATH.to_string())
                .width(width)
                .height(heigth)
                .scale(card_img_middle_base_scale()))
            .position(spawn_point);
            card_stack = card_stack.push(base_card);
        }
        Container::new(card_stack)
            .width(self.width())
            .height(self.height())
    }
}
