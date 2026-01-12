use super::GameElement;
use crate::client::AppMessage;
use crate::game_elements::hand_card::{Card, CardMessage, CardMoveState};

use iced::{Point, Size, widget::{Container, Stack, container, pin, stack}};
use indexmap::{IndexMap, map::MutableKeys};

static CARD1_PATH:&'static str = "assets/cards/1.png";
static CARD2_PATH:&'static str = "assets/cards/2.png";
static CARD3_PATH:&'static str = "assets/cards/3.png";
static CARD4_PATH:&'static str = "assets/cards/4.png";
static MULT_BASE_WIDTH_CARD_WIDTH: f32 = 0.12;
static MULT_BASE_WIDTH_CARD_HEIGHT: f32 = MULT_BASE_WIDTH_CARD_WIDTH * 1.46;
static MULT_BASE_WIDTH_CARD_STACK_OFFSET: f32 = MULT_BASE_WIDTH_CARD_HEIGHT * 0.53;

#[derive(Debug, Clone)]
pub enum HandMessage {
    CardMessage(CardMessage)
}

#[derive(Debug)]
pub struct Hand {
    window_size: Size,

    cards: IndexMap<usize, Card>,
    card_base_size: Size,
    focus_card_row_low: bool,
    top_card_id_upper: usize,
    top_card_id_lower: usize
}

impl Default for Hand {
    fn default() -> Self {
        Self {
            window_size: Size::new(300.0, 300.0),
            cards: IndexMap::from([
                (0, Card::new(0, CARD1_PATH, Size::new(154.0, 225.0))),
                (1, Card::new(1, CARD3_PATH, Size::new(154.0, 225.0))),
                (2, Card::new(2, CARD2_PATH, Size::new(154.0, 225.0))),
                (3, Card::new(3, CARD1_PATH, Size::new(154.0, 225.0))),
                (4, Card::new(4, CARD4_PATH, Size::new(154.0, 225.0))),
                (5, Card::new(5, CARD2_PATH, Size::new(154.0, 225.0))),
                (6, Card::new(6, CARD1_PATH, Size::new(154.0, 225.0))),
                (7, Card::new(7, CARD3_PATH, Size::new(154.0, 225.0))),
                (8, Card::new(8, CARD2_PATH, Size::new(154.0, 225.0))),
                (9, Card::new(9, CARD1_PATH, Size::new(154.0, 225.0))),
                (10, Card::new(10, CARD4_PATH, Size::new(154.0, 225.0))),
                (11, Card::new(11, CARD2_PATH, Size::new(154.0, 225.0))),
                (12, Card::new(12, CARD1_PATH, Size::new(154.0, 225.0))),
                (13, Card::new(13, CARD4_PATH, Size::new(154.0, 225.0))),
                (14, Card::new( 14, CARD2_PATH, Size::new(154.0, 225.0))),
                (15, Card::new(15, CARD1_PATH, Size::new(154.0, 225.0))),
                (16, Card::new(16, CARD3_PATH, Size::new(154.0, 225.0))),
            ]),
            card_base_size: Size::new(154.0, 225.0),
            focus_card_row_low: true,
            top_card_id_upper: 100, // Impossible to reach
            top_card_id_lower: 100  // Impossible to reach   
        }    
    }
}

impl Hand {

    fn get_card_mut(&mut self, id: usize) -> &mut Card {
        self.cards.get_mut(&id).unwrap()
    }

    fn get_card_width(&self) -> f32 {
        self.window_size.width * MULT_BASE_WIDTH_CARD_WIDTH
    }

    fn get_card_height(&self) -> f32 {

        self.window_size.width * MULT_BASE_WIDTH_CARD_HEIGHT
    }

    fn get_card_spawn_point_upper_row(&self) -> Point {

        let max_row_len: f32 = 4.0 * self.card_base_size.width;

        let row_y_offset: f32 = self.get_hand_row_distance();

        let mut row_x_offset: f32 = 0.0;
        if self.cards.len() > 10 {
            let cards_in_row: usize = self.cards.len() - 10;
            let row_len: f32 = (cards_in_row as f32) * self.card_base_size.width / 3.0 +
                                     self.card_base_size.width * (2.0/3.0);
            row_x_offset = (max_row_len - row_len) / 2.0;
        }

        Point::new(row_x_offset, row_y_offset)
    }

    fn get_card_spawn_point_lower_row(&self) -> Point {

        let max_row_len: f32 = 4.0 * self.card_base_size.width;

        let cards_in_row: usize = std::cmp::min(self.cards.len(), 10);
        let row_len: f32 = (cards_in_row as f32) * self.card_base_size.width / 3.0 +
                           self.card_base_size.width * (2.0/3.0);
        let row_x_offset: f32 = (max_row_len - row_len) / 2.0;
        let row_y_offset: f32 = 0.0;

        Point::new(row_x_offset, row_y_offset)
    }

    fn get_card_ids(&self) -> Vec<usize> {
        let mut card_ids: Vec<usize> = vec!();
        for card in self.cards.iter() {
            card_ids.push(*card.0);
        }
        card_ids
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
}

impl GameElement for Hand {

    type HigherMessage = AppMessage;
    type OwnMessage = HandMessage;

    fn convert_msg(msg: AppMessage) -> HandMessage {
        match msg {
            AppMessage::HandMessage(hand_msg) => hand_msg,
            _ => panic!("Converting AppMessage to HandMessage was not possible")
        }
    }

    fn convert_to_app_message(msg: HandMessage) -> AppMessage {
        AppMessage::HandMessage(msg)
    }

    fn update_with_msg(&mut self, msg: HandMessage) {
        let card_msg = Card::convert_msg(msg);
        match card_msg {
            CardMessage::CardHovered(id) => {
                if self.cards.len() > 10 && self.get_card_ids()[..self.cards.len() - 10]
                                                                .contains(&id) {
                    self.focus_card_row_low = false;
                    self.top_card_id_upper = id;
                } else {
                    self.focus_card_row_low = true;
                    self.top_card_id_lower = id;
                }
            }
            _ => ()
        }
        self.get_card_mut(card_msg.get_id()).update_with_msg(card_msg);
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
            /* Sometimes on_exit for a viewed card won't register
               and won't send the CardNotHoverd msg.
               To ensure that an unhovered card is not sticking up
               following if-statement checks for this unwanted state
               and instead sends the msg itself.
            */
            if *id != self.top_card_id_lower &&
                    *id != self.top_card_id_upper &&
                    card.offset != 0.0 &&
                    card.move_state != CardMoveState::MovingDown {
                card.update_with_msg(CardMessage::CardNotHovered(*id));
            }
        };
    }

    fn view<'a>(&self) -> Container<'a, AppMessage> {

        // Create a stack for the whole hand and another two for the upper/lower row.
        let mut card_stack: Stack<'_, AppMessage> = stack!();
        let mut card_stack_upper: Stack<'_, AppMessage> = stack!();
        let mut card_stack_lower: Stack<'_, AppMessage> = stack!();

        // Push all cards in self.cards to their row.
        let mut x_pos: f32 = 0.0;
        let y_pos: f32 = 0.0;
        let x_pos_offset: f32 = self.get_hand_width_offset();
        let y_pos_offset: f32 = self.get_hand_height_offset();
        let mut move_card_stack_lower = true;
        if self.cards.len() > 10 {
            move_card_stack_lower = false;
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

            x_pos = x_pos + card.size.width / 3.0;

            // Switch the current row.
            if self.cards.len() > 10 && i + 1 == self.cards.len() - 10 {
                x_pos = 0.0;
                push_lower = false;
                move_card_stack_lower = true;
            }
        }

        // Add the upper/lower row to the whole hand.
        if self.focus_card_row_low {
            card_stack = card_stack.push(pin(card_stack_upper)
                                            .position(self.get_card_spawn_point_upper_row()));
            card_stack = card_stack.push(pin(card_stack_lower)
                                            .position(self.get_card_spawn_point_lower_row()));
        } else {
            card_stack = card_stack.push(pin(card_stack_lower)
                                            .position(self.get_card_spawn_point_lower_row()));
            card_stack = card_stack.push(pin(card_stack_upper)
                                            .position(self.get_card_spawn_point_upper_row()));
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
