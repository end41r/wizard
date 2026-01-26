use super::GameElement;
use crate::client::AppMessage;
use crate::game_elements::{AnimationCore, AnimationState};
use crate::game_elements::hand_card::hand_card::{Card, CardMessage};
use super::{AnimationEndSensor, AnimationStarter};

use std::num::NonZero;
use iced::{Point, Size, widget::{Container, Stack, container, pin, stack}};
use indexmap::{IndexMap, map::MutableKeys};

static CARD1_PATH:&'static str = "assets/cards/1.png";
static CARD2_PATH:&'static str = "assets/cards/2.png";
static CARD3_PATH:&'static str = "assets/cards/3.png";
static CARD4_PATH:&'static str = "assets/cards/4.png";
static MULT_BASE_WIDTH_CARD_WIDTH: f32 = 0.12;
static MULT_BASE_WIDTH_CARD_HEIGHT: f32 = MULT_BASE_WIDTH_CARD_WIDTH * 1.46;
static MULT_BASE_WIDTH_CARD_STACK_OFFSET: f32 = MULT_BASE_WIDTH_CARD_HEIGHT * 0.43;

#[derive(Debug, Clone)]
pub enum HandMessage {
    CardMessage(CardMessage),
    DrawCards(Vec<Card>),
    HideCards,
    ShowCards
}

#[derive(Debug)]
pub struct Hand {
    window_size: Size,

    cards: IndexMap<usize, Card>,
    card_base_size: Size,
    hovered_card_row_low: bool,
    hovered_card_id: usize,
    top_card_id_upper: usize,
    top_card_id_lower: usize,
    hide_animation_ticker: AnimationEndSensor<usize>,
    draw_animation_ticker: AnimationStarter<Vec<Card>>
}

impl Default for Hand {
    fn default() -> Self {
        Self {
            window_size: Size::new(300.0, 300.0),
            cards: IndexMap::from([]),
            card_base_size: Size::new(154.0, 225.0),
            hovered_card_row_low: true,
            hovered_card_id: 100,
            top_card_id_upper: 100, // Impossible to reach
            top_card_id_lower: 100,  // Impossible to reach
            hide_animation_ticker: AnimationEndSensor::new(NonZero::new(20).unwrap()),
            draw_animation_ticker: AnimationStarter::new(NonZero::new(3).unwrap())
        }    
    }
}

impl Hand {

    fn get_card_width(&self) -> f32 {
        self.window_size.width * MULT_BASE_WIDTH_CARD_WIDTH
    }

    fn get_card_height(&self) -> f32 {

        self.window_size.width * MULT_BASE_WIDTH_CARD_HEIGHT
    }

    fn get_card_spawn_point_upper_row(&self, x_pos_step: f32) -> Point {

        let max_row_len: f32 = 4.0 * self.card_base_size.width;

        let row_y_offset: f32 = self.get_hand_row_distance();

        let mut row_x_offset: f32 = 0.0;
        if self.cards.len() > 10 {
            let cards_in_row: usize = self.cards.len() - 10;
            let row_len: f32 = ((cards_in_row as f32) - 1.0) * x_pos_step +
                                     self.card_base_size.width;
            row_x_offset = (max_row_len - row_len) / 2.0;
        }

        Point::new(row_x_offset, row_y_offset)
    }

    fn get_card_spawn_point_lower_row(&self, x_pos_step: f32) -> Point {

        let max_row_len: f32 = 4.0 * self.card_base_size.width;

        let cards_in_row: usize = std::cmp::min(self.cards.len(), 10);
        let mut row_len: f32 = 0.0;
        if self.cards.len() != 0 {
            row_len = ((cards_in_row as f32) - 1.0) * x_pos_step + self.card_base_size.width;
        };
        let row_x_offset: f32 = (max_row_len - row_len) / 2.0;
        let row_y_offset: f32 = 0.0;

        Point::new(row_x_offset, row_y_offset)
    }

    fn get_card_ids(&self) -> Vec<usize> {
        let mut card_ids: Vec<usize> = vec!();
        for (id, _) in self.cards.iter() {
            card_ids.push(*id);
        }
        card_ids
    }

    pub fn get_cards(&self) -> Vec<Card> {
        let mut cards: Vec<Card> = vec!();
        for (_, card) in self.cards.iter() {
            cards.push(card.clone());
        }
        cards
    }

    pub fn build_test_cards() -> Vec<Card> {
        vec![
            Card::new(0, CARD1_PATH, Size::new(154.0, 225.0)),
            Card::new(1, CARD3_PATH, Size::new(154.0, 225.0)),
            Card::new(2, CARD2_PATH, Size::new(154.0, 225.0)),
            Card::new(3, CARD1_PATH, Size::new(154.0, 225.0)),
            Card::new(4, CARD4_PATH, Size::new(154.0, 225.0)),
            Card::new(5, CARD2_PATH, Size::new(154.0, 225.0)),
            Card::new(6, CARD1_PATH, Size::new(154.0, 225.0)),
            Card::new(7, CARD3_PATH, Size::new(154.0, 225.0)),
            Card::new(8, CARD2_PATH, Size::new(154.0, 225.0)),
            Card::new(9, CARD1_PATH, Size::new(154.0, 225.0)),
            Card::new(10, CARD4_PATH, Size::new(154.0, 225.0)),
            Card::new(11, CARD2_PATH, Size::new(154.0, 225.0)),
            Card::new(12, CARD1_PATH, Size::new(154.0, 225.0)),
            Card::new(13, CARD4_PATH, Size::new(154.0, 225.0)),
            Card::new( 14, CARD2_PATH, Size::new(154.0, 225.0)),
            Card::new(15, CARD1_PATH, Size::new(154.0, 225.0)),
            Card::new(16, CARD3_PATH, Size::new(154.0, 225.0)),
            ]
    }

    fn get_hand_width(&self) -> f32 {
        // 10 cards layered size with max width reached while left and right card at max size
        self.card_base_size.width * 3.0 +
        self.card_base_size.width * 1.1
    }

    fn get_hand_height(&self) -> f32 {
        // This does ignore size multiplication of cards sinze their dimensions move down
        // while the card is moving up so the total length of the card hand is not altered by that.
        self.card_base_size.height -  // card lower
        self.get_hand_row_distance() +  // card upper
        self.card_base_size.height * 0.15  // card offset
    }

    fn get_hand_width_offset(&self) -> f32 {
        (self.card_base_size.width * 1.1 - self.card_base_size.width) / 2.0
    }

    fn get_hand_height_offset(&self) -> f32 {
        self.get_hand_height() - self.card_base_size.height
    }

    fn get_hand_row_distance(&self) -> f32 {
        -self.window_size.width * MULT_BASE_WIDTH_CARD_STACK_OFFSET
    }

    fn change_cards(&mut self, cards: Vec<Card>) {
        self.cards.clear();
        for card in cards.iter() {
            self.cards.insert(card.id, card.clone());
        }
    }

    fn update_cards_with_msg(&mut self, msg: CardMessage) {
        for (_, card) in self.cards.iter_mut() {
            card.update_with_msg(msg.clone());
        }
    }
}

impl GameElement for Hand {

    type OwnMessage = HandMessage;

    fn convert_to_app_message(msg: HandMessage) -> AppMessage {
        AppMessage::HandMessage(msg)
    }

    fn update_with_msg(&mut self, msg: HandMessage) {
        match msg {
            HandMessage::CardMessage(card_msg) => {
                match card_msg {
                    CardMessage::Hovered(id) => {
                        if !self.hide_animation_ticker.active() {
                            self.update_cards_with_msg(card_msg);
                            self.hovered_card_id = id;
                            if self.cards.len() > 10 && self.get_card_ids()[..self.cards.len() - 10]
                                                                            .contains(&id) {
                                self.hovered_card_row_low = false;
                                self.top_card_id_upper = id;
                            } else {
                                self.hovered_card_row_low = true;
                                self.top_card_id_lower = id;
                            }
                        }
                    }
                    CardMessage::Played(id) => {
                        if !self.draw_animation_ticker.active() {
                            self.update_cards_with_msg(card_msg);
                            self.update_with_msg(HandMessage::HideCards);
                            self.hide_animation_ticker.start(Some(id));
                        }
                    }
                    _ => {self.update_cards_with_msg(card_msg);}
                }
            }
            HandMessage::HideCards => {
                for (id, card) in self.cards.iter_mut() {
                    if !card.play_animation.active() {
                        card.update_with_msg(CardMessage::Hide(*id));
                    }
                }
            }
            HandMessage::ShowCards => {
                for (id, card) in self.cards.iter_mut() {
                    card.update_with_msg(CardMessage::Show(*id));
                }
            }
            HandMessage::DrawCards(cards) => {
                if !self.draw_animation_ticker.active() {  // redrawing while drawing causes bugs
                    self.change_cards(cards.clone());
                    self.hovered_card_row_low = true;
                    self.hovered_card_id = 1000;  // Impossible to reach
                    self.top_card_id_lower = 1000;  // Impossible to reach
                    self.top_card_id_upper = 1000;  // Impossible to reach
                    self.update_size(self.window_size);
                    self.draw_animation_ticker.start(None, self.cards.len());
                }
            }
        }
    }

    fn update_size(&mut self, window_size: Size) {
        self.window_size = window_size;
        self.card_base_size = Size::new(self.get_card_width(),
                                        self.get_card_height());
        for (_, card) in self.cards.iter_mut2() {
            card.update_size(self.card_base_size);
        };
    }
    
    fn update_animations(&mut self) {
        for (id, card) in self.cards.iter_mut2() {
            card.update_animations();
            // Sometimes on_exit for a viewed card won't register
            // and won't send the CardNotHoverd msg.
            // To ensure that an unhovered card is not sticking up all the time
            // following if-statement tries to check on this unwanted state
            // and instead sends the msg itself.
            if *id != self.hovered_card_id &&
                    card.hover_animation.get_offset() != 0.0 &&
                    card.hover_animation.animation_state != AnimationState::Reversing {
                card.update_with_msg(CardMessage::NotHovered(*id));
            }
        };
        if self.hide_animation_ticker.check(|h: &mut AnimationEndSensor<usize>| {
            let key: &usize = h.content().unwrap();
            self.cards.shift_remove(key);
        }) {
            self.update_with_msg(HandMessage::ShowCards);
        };
        self.draw_animation_ticker.check(|d: &mut AnimationStarter<Vec<Card>>| {
            let id = self.cards[d.cycle()].id;
            self.cards[d.cycle()].update_with_msg(CardMessage::Draw(id));
        });
    }

    fn view<'a>(&self) -> Container<'a, AppMessage> {

        // Create a stack for the whole hand and another two for the upper/lower row.
        let mut card_stack: Stack<'_, AppMessage> = stack!();
        let mut card_stack_upper: Stack<'_, AppMessage> = stack!();
        let mut card_stack_lower: Stack<'_, AppMessage> = stack!();

        // Push all cards in self.cards to their row.
        let mut x_pos: f32 = 0.0;
        let mut x_pos_step: f32 = 0.0;
        let mut x_pos_step_upper: f32 = 0.0;
        let mut x_pos_step_lower: f32 = 0.0;
        let y_pos: f32 = 0.0;
        let x_pos_offset: f32 = self.get_hand_width_offset();
        let y_pos_offset: f32 = self.get_hand_height_offset();
        let mut move_card_stack_lower = true;
        if self.cards.len() > 10 {
            x_pos_step = match self.cards.len() {
                11..=14 => self.card_base_size.width,
                _ => self.card_base_size.width / 3.0 + (4.0 * self.card_base_size.width - ((self.cards.len() as f32 - 11.0) * self.card_base_size.width / 3.0 + self.card_base_size.width)) / (self.cards.len() as f32 - 11.0)
            };
            x_pos_step_upper = x_pos_step;
            move_card_stack_lower = false;
        } else {
            x_pos_step = match self.cards.len() {
                1..=4 => self.card_base_size.width,
                _ => self.card_base_size.width / 3.0 + (4.0 * self.card_base_size.width - ((self.cards.len() as f32 - 1.0) * self.card_base_size.width / 3.0 + self.card_base_size.width)) / (self.cards.len() as f32 - 1.0)
            };
            x_pos_step_lower = x_pos_step;
        }
        let mut push_lower = false;

        for (i, (card_id, card)) in self.cards.iter().enumerate() {

            let viewable_card: Container<'_, AppMessage>
                = card.view_and_move(x_pos + x_pos_offset, y_pos + y_pos_offset);

            if move_card_stack_lower {
                if push_lower {
                    card_stack_lower = card_stack_lower.push_under(viewable_card)
                } else {
                    card_stack_lower = card_stack_lower.push(viewable_card)
                }
            } else {
                if push_lower {
                    card_stack_upper = card_stack_upper.push_under(viewable_card)
                } else {
                    card_stack_upper = card_stack_upper.push(viewable_card)
                }
            }
            // The top card of the current row is reached.
            if (!move_card_stack_lower && *card_id == self.top_card_id_upper) ||
            (move_card_stack_lower && *card_id == self.top_card_id_lower) {
                    push_lower = true;
            }

            x_pos = x_pos + x_pos_step;

            // Switch the current row.
            if self.cards.len() > 10 && i + 1 == self.cards.len() - 10 {
                x_pos = 0.0;
                x_pos_step = self.card_base_size.width / 3.0;
                x_pos_step_lower = x_pos_step;
                push_lower = false;
                move_card_stack_lower = true;
            }
        }

        // Add the upper/lower row to the whole hand.
        if self.hovered_card_row_low {
            card_stack = card_stack.push(pin(card_stack_upper)
                                            .position(self.get_card_spawn_point_upper_row(x_pos_step_upper)));
            card_stack = card_stack.push(pin(card_stack_lower)
                                            .position(self.get_card_spawn_point_lower_row(x_pos_step_lower)));
        } else {
            card_stack = card_stack.push(pin(card_stack_lower)
                                            .position(self.get_card_spawn_point_lower_row(x_pos_step_lower)));
            card_stack = card_stack.push(pin(card_stack_upper)
                                            .position(self.get_card_spawn_point_upper_row(x_pos_step_upper)));
        }
        container(card_stack).width(self.get_hand_width()).height(self.get_hand_height())
    }

    fn view_and_move<'a>(&self, x: f32, y: f32) -> Container<'a, AppMessage> {
        container(pin(
            self.view()).position(Point::new(x, y))
        )
        .width(self.get_hand_width()).height(self.get_hand_height())
    }
}
