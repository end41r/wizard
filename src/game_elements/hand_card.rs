use iced::{Point, Size, mouse::Interaction, widget::{Container, MouseArea, container, image, pin}};
use super::GameElement;
use crate::game_elements::hand::{Hand, HandMessage};
use crate::client::AppMessage;

#[derive(Debug, Clone)]
pub enum CardMessage {
    CardPlayed(usize),
    CardHovered(usize),
    CardNotHovered(usize),
}

#[derive(PartialEq, Debug)]
pub enum CardMoveState {
    MovingUp,
    MovingDown,
    NotMoving
}

impl CardMessage {
    pub fn get_id(&self) -> usize{
        match self {
            CardMessage::CardHovered(id) => *id,
            CardMessage::CardNotHovered(id) => *id,
            CardMessage::CardPlayed(id) => *id
        }
    }
}

#[derive(Debug)]
pub struct Card {
    id: usize,
    img_path: &'static str,
    pub offset: f32,
    pub size: Size,
    size_mult: f32,
    pub move_state: CardMoveState,
}

impl Card {

    pub fn new(id: usize, img_path: &'static str, size: Size) -> Self {
        Self {
            id: id,
            img_path: img_path,
            size: size,
            size_mult: 1.0,
            offset: 0.0,
            move_state: CardMoveState::NotMoving
        }
    }

    fn move_card_up(&mut self) {
        let max_card_offset: f32 = self.size.height * 0.15;
        if self.move_state == CardMoveState::MovingUp && self.offset <= max_card_offset {
            self.size_mult += 0.02;
            self.offset += max_card_offset * 0.2;
        }
        else if self.move_state != CardMoveState::MovingDown {
            self.move_state = CardMoveState::NotMoving;
        }
    }

    fn move_card_down(&mut self) {
        let max_card_offset: f32 = self.size.height * 0.15;
        if self.move_state == CardMoveState::MovingDown && self.offset > 0.0 {
            self.size_mult -= 0.02;
            self.offset -= max_card_offset * 0.20;
        }
        else if self.move_state != CardMoveState::MovingUp {
            self.move_state = CardMoveState::NotMoving;
            // Correcting floating point error
            self.size_mult = 1.0;
            self.offset = 0.0;
        }
    }
}

impl GameElement for Card {

    type HigherMessage = HandMessage;
    type OwnMessage = CardMessage;

    fn convert_msg(msg: HandMessage) -> CardMessage {
        match msg {
            HandMessage::CardMessage(card_msg) => card_msg
        }
    }

    fn convert_to_app_message(msg: CardMessage) -> AppMessage {
        Hand::convert_to_app_message(HandMessage::CardMessage(msg))
    }

    fn update_with_msg(&mut self, msg: CardMessage) {
        match msg {
            CardMessage::CardHovered(_) => {
                self.move_state = CardMoveState::MovingUp;
            }
            CardMessage::CardPlayed(id) => {
                println!("Card with id {} played!", id);
            }
            CardMessage::CardNotHovered(_) => {
                self.move_state = CardMoveState::MovingDown;
            }
        }
    }

    fn update_animations(&mut self) {
        match self.move_state {
            CardMoveState::MovingUp => {
                self.move_card_up();
            }
            CardMoveState::MovingDown => {
                self.move_card_down();
            }
            CardMoveState::NotMoving => {
                ();
            }
        }
    }

    fn update_size(&mut self, window_size: Size) {
        self.size = window_size;
    }

    fn view<'a>(&self) -> Container<'a, AppMessage> {
        container(MouseArea::new(image(self.img_path)
            .width(self.size.width * self.size_mult)
            .height(self.size.height * self.size_mult))
            .on_double_click(Card::convert_to_app_message(CardMessage::CardPlayed(self.id)))
            .on_enter(Card::convert_to_app_message(CardMessage::CardHovered(self.id)))
            .on_exit(Card::convert_to_app_message(CardMessage::CardNotHovered(self.id)))
            .interaction(Interaction::Pointer)
        )
    }

    fn view_and_move<'a>(&self, x: f32, y: f32) -> Container<'a, AppMessage> {
        container(pin(self.view())
            .position(
                Point::new(
                    x + (self.size.width - self.size.width * self.size_mult) / 2.0,
                    y - self.offset
                )
            )
        )
    }
}
