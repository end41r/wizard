mod hand_card;

use crate::animation::{
    animation_end_sensor::AnimationEndSensor, animation_starter::AnimationStarter,
};
use crate::client::AppMessage;
use crate::gameplay_ui::hand::hand_card::{CardMessage, ViewableCard};
use crate::ui_element_traits::*;

use iced::{
    widget::{container, pin, stack, Container, Pin, Stack},
    Point, Size,
};
use indexmap::{map::MutableKeys, IndexMap};
use std::num::NonZero;

use crate::gameplay_ui::hand::hand_card::CARD_HEIGHT_MULT_WITH_WINDOW_WIDTH;

static CARD1_PATH: &str = "assets/cards/back.png";
static CARD2_PATH: &str = "assets/cards/diamond.png";
static CARD3_PATH: &str = "assets/cards/heart.png";
static CARD4_PATH: &str = "assets/cards/spade.png";

// Adjust this arbitrary value to manipulate the width of the hand relative to its size,
// but be careful that the cards don't go out of screen.
// If you want to manipulate the total hand size
// change the value of hand_card::CARD_WIDTH_MULT_WITH_WINDOW_WIDTH.
static CARD_STEP_MULT_WITH_CARD_WIDTH: f32 = 1.0 / 3.0;
// Adjust this arbitrary value to manipulate the width of the hand,
static CARD_ROW_STEP_MULT_WITH_WINDOW_WIDTH: f32 = CARD_HEIGHT_MULT_WITH_WINDOW_WIDTH * 0.43;

#[derive(Debug, Clone)]
pub enum HandMessage {
    CardMessage(CardMessage),
    DrawCards(Vec<ViewableCard>),
    HideCards,
    ShowCards,
    ShowPlayableStatus(bool),
}

#[derive(Debug)]
pub struct ViewableHand {
    window_size: Size,

    pub cards: IndexMap<usize, ViewableCard>,
    hovered_card_row_low: bool,
    hovered_card_id: Option<usize>,
    top_card_id_upper: Option<usize>,
    top_card_id_lower: Option<usize>,
    hide_animation_end_sensor: AnimationEndSensor<usize>,
    // AI-Usage: Claude.ai for the idea of passing a union type for a generic
    //           where the type doesn't matter.
    draw_animation_starter: AnimationStarter<()>,
}

impl ViewableHand {
    pub fn new(window_size: Size) -> Self {
        Self {
            window_size,
            cards: IndexMap::from([]),
            hovered_card_row_low: true,
            hovered_card_id: None,
            top_card_id_upper: None,
            top_card_id_lower: None,
            // 12 is the duration of the hide animation.
            hide_animation_end_sensor: AnimationEndSensor::new(NonZero::new(12).unwrap()),
            // 3 is the delay for the animation start.
            draw_animation_starter: AnimationStarter::new(NonZero::new(3).unwrap()),
        }
    }

    pub fn build_test_cards(window_size: Size) -> Vec<ViewableCard> {
        vec![
            ViewableCard::new(0, CARD1_PATH, window_size, true),
            ViewableCard::new(1, CARD3_PATH, window_size, false),
            ViewableCard::new(2, CARD2_PATH, window_size, true),
            ViewableCard::new(3, CARD1_PATH, window_size, true),
            ViewableCard::new(4, CARD4_PATH, window_size, true),
            ViewableCard::new(5, CARD2_PATH, window_size, true),
            ViewableCard::new(6, CARD1_PATH, window_size, true),
            ViewableCard::new(7, CARD3_PATH, window_size, false),
            ViewableCard::new(8, CARD2_PATH, window_size, true),
            ViewableCard::new(9, CARD1_PATH, window_size, true),
            ViewableCard::new(10, CARD4_PATH, window_size, true),
            ViewableCard::new(11, CARD2_PATH, window_size, true),
            ViewableCard::new(12, CARD1_PATH, window_size, true),
            ViewableCard::new(13, CARD4_PATH, window_size, false),
            ViewableCard::new(14, CARD2_PATH, window_size, true),
            ViewableCard::new(15, CARD1_PATH, window_size, true),
            ViewableCard::new(16, CARD1_PATH, window_size, true),
            ViewableCard::new(17, CARD2_PATH, window_size, true),
            ViewableCard::new(18, CARD3_PATH, window_size, true),
            ViewableCard::new(19, CARD3_PATH, window_size, true),
        ]
    }

    pub fn set_cards(&mut self, cards: Vec<ViewableCard>) {
        self.cards.clear();
        for card in cards.iter() {
            self.cards.insert(card.id, card.clone());
        }
    }

    pub fn card_ids(&self) -> Vec<usize> {
        let mut card_ids: Vec<usize> = vec![];
        for (id, _) in self.cards.iter() {
            card_ids.push(*id);
        }
        card_ids
    }

    /// The return value represents the step length between cards in a row length of 10 cards.
    fn card_minimum_step(&self) -> f32 {
        ViewableCard::width_for(self.window_size) * CARD_STEP_MULT_WITH_CARD_WIDTH
    }

    /// A card would normally be rendered anchored to the top of the hand.
    /// This function calculates the correct y position for the card.
    /// This takes in consideration the row step and the hover animation offset.
    fn card_y_offset_correction(&self) -> f32 {
        self.height() - ViewableCard::height_for(self.window_size)
    }

    fn upper_row_card_step(&self) -> f32 {
        if self.upper_row_exists() {
            match self.cards.len() {
                11..=14 => ViewableCard::width_for(self.window_size),
                _ => {
                    // A negative value is ok.
                    let top_cards_without_last: f32 = self.cards.len() as f32 - 11.0;
                    let upper_row_length: f32 = top_cards_without_last * self.card_minimum_step()
                        + ViewableCard::width_for(self.window_size);

                    self.card_minimum_step()
                        + (self.width_without_animations() - upper_row_length)
                            / top_cards_without_last
                }
            }
        } else {
            0.0 // The offset does not matter if the upper row doesn't exist, so 0.0 is fine.
        }
    }

    fn lower_row_card_step(&self) -> f32 {
        if !self.upper_row_exists() {
            match self.cards.len() {
                1..=4 => ViewableCard::width_for(self.window_size),
                _ => {
                    // A negative value is ok.
                    let cards_without_last: f32 = self.cards.len() as f32 - 1.0;
                    let upper_row_length: f32 = cards_without_last * self.card_minimum_step()
                        + ViewableCard::width_for(self.window_size);

                    self.card_minimum_step()
                        + (self.width_without_animations() - upper_row_length) / cards_without_last
                }
            }
        } else {
            self.card_minimum_step()
        }
    }

    fn upper_row_card_spawn_point(&self) -> Point {
        let max_row_len: f32 = self.width_without_animations();

        let row_y_offset: f32 = self.row_step();

        let mut row_x_offset: f32 = 0.0;
        if self.upper_row_exists() {
            // All cards are only shown within the range of the card step except one focused card.
            let row_len: f32 = ((self.upper_row_card_count() as f32) - 1.0)
                * self.upper_row_card_step()
                + ViewableCard::width_for(self.window_size);
            row_x_offset = (max_row_len - row_len) / 2.0;
        }

        Point::new(row_x_offset, row_y_offset)
    }

    fn lower_row_card_spawn_point(&self) -> Point {
        let max_row_len: f32 = self.width_without_animations();

        let cards_in_row: usize = std::cmp::min(self.cards.len(), 10);
        let mut row_len: f32 = 0.0;
        if !self.cards.is_empty() {
            // All cards are only shown within the range of the card step except one focused card.
            row_len = ((cards_in_row as f32) - 1.0) * self.lower_row_card_step()
                + ViewableCard::width_for(self.window_size);
        };
        let row_x_offset: f32 = (max_row_len - row_len) / 2.0;
        let row_y_offset: f32 = 0.0;

        Point::new(row_x_offset, row_y_offset)
    }

    fn row_step(&self) -> f32 {
        -self.window_size.width * CARD_ROW_STEP_MULT_WITH_WINDOW_WIDTH
    }

    fn upper_row_exists(&self) -> bool {
        // One row can only hold up to 10 cards.
        self.cards.len() > 10
    }

    fn upper_row_card_count(&self) -> usize {
        if self.upper_row_exists() {
            // 10 represents the number of cards in a full lower row.
            self.cards.len() - 10
        } else {
            0
        }
    }

    fn width_without_animations(&self) -> f32 {
        // The hand reaches its maximum size with 10 cards.
        self.card_minimum_step() * 9.0 +  // 9 cards only viewed within their offset
        // Only one card is fully visible (the hovered card).
        ViewableCard::width_for(self.window_size)
    }

    fn width_overflow_one_side(&self) -> f32 {
        // A card can reach a max size bigger times 1.1 via the hover animation.
        // So it may increase the hand size on one side by self.card_base_size.width * 0.05.
        ViewableCard::width_for(self.window_size) * 0.05
    }

    fn width_overflow(&self) -> f32 {
        // The total overflow that may occur on both sides.
        self.width_overflow_one_side() * 2.0
    }

    fn update_cards_with_msg(&mut self, msg: CardMessage) {
        for (_, card) in self.cards.iter_mut() {
            card.update_with_msg(msg.clone());
        }
    }
}

impl Message for ViewableHand {
    type OwnMessage = HandMessage;

    fn convert_to_app_message(msg: HandMessage) -> AppMessage {
        AppMessage::HandMessage(msg)
    }

    fn update_with_msg(&mut self, msg: HandMessage) {
        match msg {
            HandMessage::CardMessage(card_msg) => {
                match card_msg {
                    CardMessage::Hovered(id) => {
                        if !self.hide_animation_end_sensor.active() {
                            self.update_cards_with_msg(card_msg);
                            self.hovered_card_id = Some(id);
                            if self.upper_row_exists() &&
                               // AI-Usage: Claude.ai for learning how to see if value is in a vec
                               //           without the last few elements.
                               self.card_ids()[..self.upper_row_card_count()].contains(&id)
                            {
                                self.hovered_card_row_low = false;
                                self.top_card_id_upper = Some(id);
                            } else {
                                self.hovered_card_row_low = true;
                                self.top_card_id_lower = Some(id);
                            }
                        }
                    }
                    CardMessage::Played(id) => {
                        if !self.draw_animation_starter.active() {
                            self.update_cards_with_msg(card_msg);
                            self.update_with_msg(HandMessage::HideCards);
                            self.hide_animation_end_sensor.start(Some(id));
                        }
                    }
                    _ => {
                        self.update_cards_with_msg(card_msg);
                    }
                }
            }
            HandMessage::HideCards => {
                for (id, card) in self.cards.iter_mut() {
                    if !card.play_animation.moving_forward() {
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
                self.set_cards(cards.clone());
                self.hovered_card_row_low = true;
                self.hovered_card_id = None;
                self.top_card_id_lower = None;
                self.top_card_id_upper = None;
                self.draw_animation_starter.reset();
                self.hide_animation_end_sensor.reset();
                self.update_size(self.window_size);
                self.draw_animation_starter
                    .start(None, NonZero::new(self.cards.len()).unwrap());
            }
            HandMessage::ShowPlayableStatus(do_show) => {
                for (id, card) in self.cards.iter_mut() {
                    card.update_with_msg(CardMessage::ShowPlayableStatus(*id, do_show));
                }
            }
        }
    }
}

impl Animated for ViewableHand {
    fn update_animations(&mut self) {
        self.draw_animation_starter
            .check(|d: &mut AnimationStarter<()>| {
                let id: usize = self.cards[d.cycle()].id;
                self.cards[d.cycle()].update_with_msg(CardMessage::Draw(id));
            });

        for (id, card) in self.cards.iter_mut2() {
            card.update_animations();
            // Sometimes on_exit for a viewed card won't register
            // and won't send the CardNotHoverd msg.
            // To ensure that an unhovered card is not sticking up all the time
            // following if-statement tries to check for this unwanted state
            // and instead sends the msg itself.
            if self.hovered_card_id.is_some()
                && *id != self.hovered_card_id.unwrap()
                && card.hover_animation.get_offset(self.window_size) != 0.0
                && card.hover_animation.not_moving()
            {
                card.update_with_msg(CardMessage::NotHovered(*id));
            }
        }

        if self
            .hide_animation_end_sensor
            .check(|h: &mut AnimationEndSensor<usize>| {
                let key: &usize = h.content().unwrap();
                self.cards.shift_remove(key);
            })
        {
            self.update_with_msg(HandMessage::ShowCards);
        };
    }
}

impl Resizable for ViewableHand {
    fn update_size(&mut self, window_size: Size) {
        self.window_size = window_size;
        for (_, card) in self.cards.iter_mut2() {
            card.update_size(window_size);
        }
    }
    fn width(&self) -> f32 {
        self.width_without_animations() + self.width_overflow()
    }
    fn height(&self) -> f32 {
        ViewableCard::height_for(self.window_size) -  // Upper card height
        self.row_step() +  // Upper card spawn offset
        // The upper card may have an increased size via the hover animation max_offset.
        ViewableCard::height_for(self.window_size) * 0.15
    }
}

impl Viewable for ViewableHand {
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        // Create a stack for the whole hand and another two for the upper/lower row.
        let mut hand: Stack<'_, AppMessage> = stack!();
        let mut upper_card_row: Stack<'_, AppMessage> = stack!();
        let mut lower_card_row: Stack<'_, AppMessage> = stack!();

        let mut x_pos: f32 = 0.0;
        let y_pos: f32 = 0.0;

        let x_pos_offset: f32 = self.width_overflow_one_side();
        let y_pos_correction: f32 = self.card_y_offset_correction();

        let mut move_lower_card_row: bool = true;
        let mut push_lower: bool = false;

        // Calculate the step length for the first build card row.
        // A second used row will always have 10 cards,
        // so it will later use the minimum card offset.
        let mut x_pos_step: f32;

        if self.upper_row_exists() {
            x_pos_step = self.upper_row_card_step();
            move_lower_card_row = false;
        } else {
            x_pos_step = self.lower_row_card_step();
        }

        // Add all cards to their corresponding row.
        for (i, (card_id, card)) in self.cards.iter().enumerate() {
            let viewable_card: Container<'_, AppMessage> =
                card.view_and_move(x_pos + x_pos_offset, y_pos + y_pos_correction);

            if move_lower_card_row {
                if push_lower {
                    lower_card_row = lower_card_row.push_under(viewable_card)
                } else {
                    lower_card_row = lower_card_row.push(viewable_card)
                }
            } else if push_lower {
                upper_card_row = upper_card_row.push_under(viewable_card)
            } else {
                upper_card_row = upper_card_row.push(viewable_card)
            }

            // The top card of the current row is reached.
            if (self.top_card_id_upper.is_some()
                && (!move_lower_card_row && *card_id == self.top_card_id_upper.unwrap()))
                || (self.top_card_id_lower.is_some()
                    && (move_lower_card_row && *card_id == self.top_card_id_lower.unwrap()))
            {
                push_lower = true;
            }

            x_pos += x_pos_step;

            // Switch to the second row.
            let added_cards: usize = i + 1;
            if self.upper_row_exists() && added_cards == self.upper_row_card_count() {
                x_pos = 0.0;
                x_pos_step = self.card_minimum_step();
                push_lower = false;
                move_lower_card_row = true;
            }
        }

        // Adjust the spawn points of both card rows in the whole hand.
        let upper_card_row: Pin<'_, AppMessage> =
            pin(upper_card_row).position(self.upper_row_card_spawn_point());
        let lower_card_row: Pin<'_, AppMessage> =
            pin(lower_card_row).position(self.lower_row_card_spawn_point());

        // Add the upper/lower row to the whole hand.
        if self.hovered_card_row_low {
            hand = hand.push(upper_card_row);
            hand = hand.push(lower_card_row);
        } else {
            hand = hand.push(lower_card_row);
            hand = hand.push(upper_card_row);
        }

        container(hand).width(self.width()).height(self.height())
    }
}
