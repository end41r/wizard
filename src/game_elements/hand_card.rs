use iced::{Point, Size, mouse::Interaction, widget::{Container, MouseArea, container, image, pin}};
use super::{GameElement, AnimationCore, ReversableAnimation, AnimationState};
use crate::game_elements::hand::{Hand, HandMessage};
use crate::client::AppMessage;

#[derive(Debug)]
pub struct HoverAnimation {
    pub max_frame_number: usize,
    pub current_frame_number: usize,
    pub animation_state: AnimationState,

    pub max_offset: f32
}

impl HoverAnimation {
    fn new(size: Size) -> Self {
        Self {
            max_frame_number: 5,
            current_frame_number: 0,
            animation_state: AnimationState::NotMoving,
            max_offset: size.height * 0.15,
        }
    }
    pub fn update_target_max_offset(&mut self, size: Size) {  // TODO: part of trait
        self.max_offset = size.height * 0.15;
    }
    pub fn get_offset(&self) -> f32 {
        self.max_offset * 0.2 * self.current_frame_number as f32
    }
    pub fn get_size_mult(&self) -> f32 {
        1.0 + self.current_frame_number as f32 * 0.02
    }
}

impl AnimationCore for HoverAnimation {
    fn _mut_max_frame_number(&mut self) -> &mut usize {
        &mut self.max_frame_number
    }
    fn _mut_current_frame_number(&mut self) -> &mut usize {
        &mut self.current_frame_number
    }
    fn _mut_animation_state(&mut self) -> &mut AnimationState {
        &mut self.animation_state
    }
} 

impl ReversableAnimation for HoverAnimation {}

#[derive(Debug, Clone)]
pub enum CardMessage {
    CardPlayed(usize),
    CardHovered(usize),
    CardNotHovered(usize),
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
    pub size: Size,
    pub hover_animation: HoverAnimation,
}

impl Card {

    pub fn new(id: usize, img_path: &'static str, size: Size) -> Self {
        Self {
            id: id,
            img_path: img_path,
            size: size,
            hover_animation: HoverAnimation::new(size),
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
                self.hover_animation.start();
            }
            CardMessage::CardPlayed(id) => {
                println!("Card with id {} played!", id);
            }
            CardMessage::CardNotHovered(_) => {
                self.hover_animation.reverse();
            }
        }
    }

    fn update_animations(&mut self) {
        self.hover_animation.next_frame();
    }

    fn update_size(&mut self, window_size: Size) {
        self.size = window_size;
        self.hover_animation.update_target_max_offset(self.size);
    }

    fn view<'a>(&self) -> Container<'a, AppMessage> {  // TODO: combine various animation offsets via function
        container(MouseArea::new(image(self.img_path)
            .width(self.size.width * self.hover_animation.get_size_mult())
            .height(self.size.height * self.hover_animation.get_size_mult()))
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
                    x + (self.size.width - self.size.width * self.hover_animation.get_size_mult()) / 2.0,
                    y - self.hover_animation.get_offset()
                )
            )
        )
    }
}
