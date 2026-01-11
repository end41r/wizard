use iced::{Point, Size, mouse::Interaction, widget::{Container, MouseArea, Pin, Stack, container, image, pin, stack}};
use indexmap::{IndexMap, map::MutableKeys};
use crate::client::{AppMessage, GameElement};

static CARD1_PATH:&'static str = "assets/cards/1.png";
static CARD2_PATH:&'static str = "assets/cards/2.png";
static CARD3_PATH:&'static str = "assets/cards/3.png";
static CARD4_PATH:&'static str = "assets/cards/4.png";
static MULT_BASE_WIDTH_CARD_WIDTH: f32 = 0.12;
static MULT_BASE_WIDTH_CARD_HEIGHT: f32 = MULT_BASE_WIDTH_CARD_WIDTH * 1.46;
static MULT_BASE_WIDTH_CARD_STACK_OFFSET: f32 = MULT_BASE_WIDTH_CARD_HEIGHT * 0.53;

#[derive(Debug, Clone)]
pub enum HandMessage {
    CardPlayed(usize),
    CardHovered(usize),
    CardNotHovered(usize),
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
                (0, Card::new(CARD1_PATH, Size::new(154.0, 225.0))),
                (1, Card::new(CARD3_PATH, Size::new(154.0, 225.0))),
                (2, Card::new(CARD2_PATH, Size::new(154.0, 225.0))),
                (3, Card::new(CARD1_PATH, Size::new(154.0, 225.0))),
                (4, Card::new(CARD4_PATH, Size::new(154.0, 225.0))),
                (5, Card::new(CARD2_PATH, Size::new(154.0, 225.0))),
                (6, Card::new(CARD1_PATH, Size::new(154.0, 225.0))),
                (7, Card::new(CARD3_PATH, Size::new(154.0, 225.0))),
                (8, Card::new(CARD2_PATH, Size::new(154.0, 225.0))),
                (9, Card::new(CARD1_PATH, Size::new(154.0, 225.0))),
                (10, Card::new(CARD4_PATH, Size::new(154.0, 225.0))),
                (11, Card::new(CARD2_PATH, Size::new(154.0, 225.0))),
                (12, Card::new(CARD1_PATH, Size::new(154.0, 225.0))),
                (13, Card::new(CARD4_PATH, Size::new(154.0, 225.0))),
                (14, Card::new(CARD2_PATH, Size::new(154.0, 225.0))),
                (15, Card::new(CARD1_PATH, Size::new(154.0, 225.0))),
                (16, Card::new(CARD3_PATH, Size::new(154.0, 225.0))),
            ]),
            card_base_size: Size::new(154.0, 225.0),
            focus_card_row_low: true,
            top_card_id_upper: 100, // Impossible to reach
            top_card_id_lower: 100  // Impossible to reach   
        }    
    }
}

impl Hand {

    fn get_card(&self, id: usize) -> &Card {
        self.cards.get(&id).unwrap()
    }

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

    fn view_card<'a>(card_id: &usize, card: &Card, x_pos: f32, y_pos: f32) -> Pin<'a, AppMessage> {
        pin(
            MouseArea::new(image(card.img_path)
                     .width(card.size.width * card.size_mult)
                     .height(card.size.height * card.size_mult))
                     .on_double_click(AppMessage::HandMessage(HandMessage::CardPlayed(*card_id)))
                     .on_enter(AppMessage::HandMessage(HandMessage::CardHovered(*card_id)))
                     .on_exit(AppMessage::HandMessage(HandMessage::CardNotHovered(*card_id)))
                     .interaction(Interaction::Pointer)
        )
        .position(Point::new(x_pos + (card.size.width - card.size.width * card.size_mult) / 2.0,
                             y_pos-(card.offset as f32)))
    }
}

impl GameElement for Hand {

    fn update_uniques(&mut self, msg: AppMessage) {
        match msg {
            AppMessage::HandMessage(HandMessage::CardHovered(card_id)) => {
                self.get_card_mut(card_id).moving_up = CardMoveState::MovingUp;
                if self.cards.len() > 10 && self.get_card_ids()[..self.cards.len() - 10]
                                                                .contains(&card_id) {
                    self.focus_card_row_low = false;
                    self.top_card_id_upper = card_id;
                } else {
                    self.focus_card_row_low = true;
                    self.top_card_id_lower = card_id;
                }
            }
            AppMessage::HandMessage(HandMessage::CardPlayed(card_id)) => {
                println!("Card with id {} played!", card_id);
            }
            AppMessage::HandMessage(HandMessage::CardNotHovered(card_id)) => {
                self.get_card_mut(card_id).moving_up = CardMoveState::MovingDown;
            }
            _ => ()
        }
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

            let viewable_card: Pin<'_, AppMessage> = Hand::view_card(card_id, card,
                x_pos + x_pos_offset, y_pos + y_pos_offset);

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

    fn update_size(&mut self, window_size: Size) {
        self.window_size = window_size;
        self.card_base_size = Size::new(self.get_card_width(),
                                        self.get_card_height());
        for (_, card) in self.cards.iter_mut2() {
            card.size = self.card_base_size;
        };
    }
    
    fn update_animations(&mut self) {
        for card_id in self.get_card_ids().iter() {
            if self.get_card(*card_id).moving_up == CardMoveState::MovingUp {
                self.get_card_mut(*card_id).move_card_up();
            }
            if self.get_card(*card_id).moving_up == CardMoveState::MovingDown {
                self.get_card_mut(*card_id).move_card_down();
            }
            if *card_id != self.top_card_id_lower &&
                *card_id != self.top_card_id_upper &&
                self.get_card_mut(*card_id).offset != 0.0 {
                    self.get_card_mut(*card_id).move_card_down();
            }
        };
    }
}

#[derive(Debug)]
struct Card {
    img_path: &'static str,
    offset: f32,
    size: Size,
    size_mult: f32,
    moving_up: CardMoveState,
}

impl Card {

    fn new(img_path: &'static str, size: Size) -> Self{
        Card {
            img_path: img_path,
            size: size,
            size_mult: 1.0,
            offset: 0.0,
            moving_up: CardMoveState::NotMoving
        }
    }

    fn move_card_up(&mut self) {
        let max_card_offset: f32 = self.size.height * 0.15;
        if self.moving_up == CardMoveState::MovingUp && self.offset <= max_card_offset {
            self.size_mult += 0.02;
            self.offset += max_card_offset * 0.2;
        }
        else if self.moving_up != CardMoveState::MovingDown {
            self.moving_up = CardMoveState::NotMoving;
        }
    }

    fn move_card_down(&mut self) {
        let max_card_offset: f32 = self.size.height * 0.15;
        if self.moving_up == CardMoveState::MovingDown && self.offset > 0.0 {
            self.size_mult -= 0.02;
            self.offset -= max_card_offset * 0.20;
        }
        else if self.moving_up != CardMoveState::MovingUp {
            self.moving_up = CardMoveState::NotMoving;
            // Correcting floating point error
            self.size_mult = 1.0;
            self.offset = 0.0;
        }
    }
}

#[derive(PartialEq, Debug)]
enum CardMoveState {
    MovingUp,
    MovingDown,
    NotMoving
}