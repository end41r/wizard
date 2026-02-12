pub mod deck_card;
pub mod trump_card;

use crate::{
    animation::AnimationStarter,
    api::{CARD_BACK_PATH, Card},
    client::{AppMessage, TaskBatcher},
    gameplay_ui::{
        card_img_table_base_scale, card_width_middle,
        hand::ViewableHand,
        table::{
            HandMessage, middle::{
                TableMiddleMessage, ViewableTableMiddle, card_deck::{deck_card::ViewableDeckCard, trump_card::ViewableTrumpCard}
            }
        },
    },
    ui_element_traits::*,
};
use iced::{
    widget::{image, pin, stack, Container},
    Point, Size, Task,
};

type TrumpCard = Card;

#[derive(Debug, Clone)]
pub enum CardDeckMessage {
    Deal(usize, Option<TrumpCard>),
    Shuffle,
    ClearDeckCards,
    AddDeckCard(usize),
}

pub struct ViewableCardDeck {
    window_size: Size,
    show_base_card: bool,
    trump_card: Option<ViewableTrumpCard>,
    deck_cards: Vec<ViewableDeckCard>,
    deal_card_animation_starter: AnimationStarter,
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
                ViewableCardDeck::convert_msg(CardDeckMessage::AddDeckCard(0)),
            ),
        };
        viewable_card_deck
            .deal_card_animation_starter
            .on_all_ended(ViewableCardDeck::convert_msg(
                CardDeckMessage::ClearDeckCards,
            ));
        viewable_card_deck
    }
}

impl Message for ViewableCardDeck {
    type OwnMessage = CardDeckMessage;
    fn convert_msg(msg: Self::OwnMessage) -> AppMessage {
        ViewableTableMiddle::convert_msg(TableMiddleMessage::CardDeckMessage(msg))
    }
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
                return ViewableHand::convert_msg_to_task(HandMessage::DrawCards(
                    ViewableHand::build_test_cards(self.window_size),
                ));
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
        ViewableDeckCard::height_for(self.window_size)
    }
    fn width(&self) -> f32 {
        ViewableDeckCard::width_for(self.window_size)
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
            let img = image(CARD_BACK_PATH.to_string())
                .width(self.width() * card_img_table_base_scale())
                .height(self.height() * card_img_table_base_scale());
            card_stack = card_stack.push(pin(img).position(Point::new(
                card_width_middle(self.window_size) / 6.0,
                card_width_middle(self.window_size) / 6.0,
            )));
        }
        for card in self.deck_cards.iter() {
            card_stack = card_stack.push(card.view_and_move(
                card_width_middle(self.window_size) / 6.0 + card.offset().x,
                card_width_middle(self.window_size) / 6.0 + card.offset().y,
            ));
        }
        if self.trump_card.is_some() {
            card_stack = card_stack.push(self.trump_card.as_ref().unwrap().view_and_move(
                card_width_middle(self.window_size) / 6.0,
                card_width_middle(self.window_size) / 6.0,
            ))
        }
        Container::new(card_stack)
            .width(self.width())
            .height(self.height())
    }
}
