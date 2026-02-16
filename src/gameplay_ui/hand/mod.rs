pub mod hand_card;

use crate::animation::AnimationStarter;
use crate::client::{AppMessage, TaskBatcher};
use crate::gamelogic::round::random_card;
use crate::gameplay_ui::hand::hand_card::{CardMessage, ViewableHandCard};
use crate::gameplay_ui::{card_column_step_hand, card_row_step_hand, GameViewMessage};
use crate::ui_element_traits::*;

use iced::{
    widget::{container, pin, stack, Container, Pin, Stack},
    Point, Size, Task,
};
use indexmap::{map::MutableKeys, IndexMap};

#[derive(Debug, Clone)]
pub enum HandMessage {
    CardMessage(CardMessage),
    DeleteCard(usize),
    DrawCards(Vec<ViewableHandCard>),
    HideCards,
    ShowCards,
    ShowPlayableStatus(bool),
}

impl Message for HandMessage {
    fn convert_msg_from(msg: Self) -> AppMessage {
        GameViewMessage::convert_msg_from(GameViewMessage::HandMessage(msg))
    }
}

#[derive(Debug)]
pub struct ViewableHand {
    window_size: Size,

    pub cards: IndexMap<usize, ViewableHandCard>,
    hovered_card_row_low: bool,
    allow_hover: bool,
    hovered_card_id: Option<usize>,
    played_card_id: Option<usize>,
    top_card_id_upper: Option<usize>,
    top_card_id_lower: Option<usize>,
    // AI-Usage: Claude.ai for the idea of passing a union type for a generic
    //           where the type doesn't matter.
    draw_animation_starter: AnimationStarter<CardMessage, CardMessage>,
}

impl ViewableHand {
    pub fn new(window_size: Size) -> Self {
        Self {
            window_size,
            cards: IndexMap::from([]),
            hovered_card_row_low: true,
            allow_hover: true,
            hovered_card_id: None,
            played_card_id: None,
            top_card_id_upper: None,
            top_card_id_lower: None,
            // 3 is the delay for the animation start.
            draw_animation_starter: AnimationStarter::new(3, 10, CardMessage::Draw(0)),
        }
    }

    /// This function is only for testing and may return impossible dupes of cards.
    pub fn build_test_cards(window_size: Size) -> Vec<ViewableHandCard> {
        vec![
            ViewableHandCard::new(0, random_card(), window_size, true),
            ViewableHandCard::new(1, random_card(), window_size, false),
            ViewableHandCard::new(2, random_card(), window_size, true),
            ViewableHandCard::new(3, random_card(), window_size, true),
            ViewableHandCard::new(4, random_card(), window_size, true),
            ViewableHandCard::new(5, random_card(), window_size, true),
            ViewableHandCard::new(6, random_card(), window_size, true),
            ViewableHandCard::new(7, random_card(), window_size, false),
            ViewableHandCard::new(8, random_card(), window_size, true),
            ViewableHandCard::new(9, random_card(), window_size, true),
            ViewableHandCard::new(10, random_card(), window_size, true),
            ViewableHandCard::new(11, random_card(), window_size, true),
            ViewableHandCard::new(12, random_card(), window_size, true),
            ViewableHandCard::new(13, random_card(), window_size, false),
            ViewableHandCard::new(14, random_card(), window_size, true),
            ViewableHandCard::new(15, random_card(), window_size, true),
            ViewableHandCard::new(16, random_card(), window_size, true),
            ViewableHandCard::new(17, random_card(), window_size, true),
            ViewableHandCard::new(18, random_card(), window_size, true),
            ViewableHandCard::new(19, random_card(), window_size, true),
        ]
    }

    pub fn set_cards(&mut self, cards: Vec<ViewableHandCard>) {
        self.cards.clear();
        for card in cards.iter() {
            self.cards.insert(card.id(), card.clone());
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
        card_column_step_hand(ViewableHandCard::width_for(self.window_size))
    }

    /// A card would normally be rendered anchored to the top of the hand.
    /// This function calculates the correct y position for the card.
    /// This takes in consideration the row step and the hover animation offset.
    fn card_y_offset_correction(&self) -> f32 {
        self.height() - ViewableHandCard::height_for(self.window_size)
    }

    fn upper_row_card_step(&self) -> f32 {
        if self.upper_row_exists() {
            match self.cards.len() {
                11..=14 => ViewableHandCard::width_for(self.window_size),
                _ => {
                    // A negative value is ok.
                    let top_cards_without_last: f32 = self.cards.len() as f32 - 11.0;
                    let upper_row_length: f32 = top_cards_without_last * self.card_minimum_step()
                        + ViewableHandCard::width_for(self.window_size);

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
                1..=4 => ViewableHandCard::width_for(self.window_size),
                _ => {
                    // A negative value is ok.
                    let cards_without_last: f32 = self.cards.len() as f32 - 1.0;
                    let upper_row_length: f32 = cards_without_last * self.card_minimum_step()
                        + ViewableHandCard::width_for(self.window_size);

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
                + ViewableHandCard::width_for(self.window_size);
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
                + ViewableHandCard::width_for(self.window_size);
        };
        let row_x_offset: f32 = (max_row_len - row_len) / 2.0;
        let row_y_offset: f32 = 0.0;

        Point::new(row_x_offset, row_y_offset)
    }

    fn row_step(&self) -> f32 {
        -card_row_step_hand(self.window_size)
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
        ViewableHandCard::width_for(self.window_size)
    }

    fn width_overflow_one_side(&self) -> f32 {
        // A card can reach a max size bigger times 1.1 via the hover animation.
        // So it may increase the hand size on one side by self.card_base_size.width * 0.05.
        ViewableHandCard::width_for(self.window_size) * 0.05
    }

    fn width_overflow(&self) -> f32 {
        // The total overflow that may occur on both sides.
        self.width_overflow_one_side() * 2.0
    }

    fn update_cards_with_msg(&mut self, msg: CardMessage) -> Task<AppMessage> {
        let mut tb = TaskBatcher::new();
        for (_, card) in self.cards.iter_mut() {
            tb.push(card.update_with_msg(msg.clone()))
        }
        tb.batch()
    }
}

impl Notifiable for ViewableHand {
    type OwnMessage = HandMessage;

    fn update_with_msg(&mut self, msg: HandMessage) -> Task<AppMessage> {
        let mut tb = TaskBatcher::new();
        match msg {
            HandMessage::CardMessage(card_msg) => {
                match card_msg {
                    CardMessage::Hovered(id) => {
                        if self.allow_hover {
                            tb.push(self.update_cards_with_msg(card_msg));
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
                        self.played_card_id = Some(id);
                        tb.push(self.update_cards_with_msg(card_msg));
                        tb.push(HandMessage::HideCards.convert_msg_to_task());
                    }
                    _ => {
                        tb.push(self.update_cards_with_msg(card_msg));
                    }
                }
            }
            HandMessage::HideCards => {
                self.allow_hover = false;
                for (id, card) in self.cards.iter_mut() {
                    if self.played_card_id.is_some() && self.played_card_id.unwrap() != *id {
                        tb.push(card.update_with_msg(CardMessage::Hide(*id)));
                    }
                }
                self.played_card_id = None;
            }
            HandMessage::DeleteCard(id) => {
                self.cards.shift_remove(&id);
                tb.push(HandMessage::ShowCards.convert_msg_to_task());
            }
            HandMessage::ShowCards => {
                self.allow_hover = true;
                for (id, card) in self.cards.iter_mut() {
                    tb.push(card.update_with_msg(CardMessage::Show(*id)));
                }
            }
            HandMessage::DrawCards(cards) => {
                self.set_cards(cards.clone());
                self.hovered_card_row_low = true;
                self.hovered_card_id = None;
                self.top_card_id_lower = None;
                self.top_card_id_upper = None;
                self.update_size(self.window_size);
                self.draw_animation_starter.start(self.cards.len());
            }
            HandMessage::ShowPlayableStatus(do_show) => {
                for (id, card) in self.cards.iter_mut() {
                    tb.push(card.update_with_msg(CardMessage::ShowPlayableStatus(*id, do_show)));
                }
            }
        }
        tb.batch()
    }
}

impl Animated for ViewableHand {
    fn update_animations(&mut self) -> Task<AppMessage> {
        let mut tb = TaskBatcher::new();
        tb.push(self.draw_animation_starter.next_frame());

        for (_, card) in self.cards.iter_mut2() {
            tb.push(card.update_animations());
        }
        tb.batch()
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
        ViewableHandCard::height_for(self.window_size) -  // Upper card height
        self.row_step() +  // Upper card spawn offset
        // The upper card may have an increased size via the hover animation max_offset.
        ViewableHandCard::height_for(self.window_size) * 0.15
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
