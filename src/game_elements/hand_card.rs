use iced::{Point, Size, mouse::Interaction, widget::{Container, MouseArea, container, image, pin, stack}};
use super::{GameElement, AnimationCore, ReversableAnimation, AnimationState};
use crate::game_elements::{BasicAnimation, RepeatingAnimation, hand::{Hand, HandMessage}};
use crate::client::AppMessage;

#[derive(Debug, Clone)]
pub enum CardMessage {
    Played(usize),
    Hovered(usize),
    NotHovered(usize),
    Remove(usize)
}

impl CardMessage {
    pub fn get_id(&self) -> usize {
        match self {
            CardMessage::Hovered(id) => *id,
            CardMessage::NotHovered(id) => *id,
            CardMessage::Played(id) => *id,
            CardMessage::Remove(id) => *id
        }
    }
}

#[derive(Debug, Clone)]
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
    pub fn update_target_max_offset(&mut self, size: Size) {
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
pub struct RemoveAnimation {
    pub max_frame_number: usize,
    pub current_frame_number: usize,
    pub animation_state: AnimationState,

    pub opacity: f32,
    pub contraction:f32
}

impl RemoveAnimation {
    fn new() -> Self {
        Self {
            max_frame_number: 20,
            current_frame_number: 0,
            animation_state: AnimationState::NotMoving,
            opacity: 1.0,
            contraction: 1.0
        }
    }

    fn get_opacity(&self) -> f32 {
        self.opacity - self.current_frame_number as f32 * (1.0 / self.max_frame_number as f32)
    }

    fn get_contraction(&self) -> f32 {
        self.contraction * (1.0 - 0.125 * self.current_frame_number as f32)
    }
}

impl AnimationCore for RemoveAnimation {
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

impl BasicAnimation for RemoveAnimation {}

#[derive(Debug, Clone)]
pub struct FocusAnimation {
    pub max_frame_number: usize,
    pub current_frame_number: usize,
    pub animation_state: AnimationState,
    pub img_path: &'static str,
}

impl FocusAnimation {
    fn new() -> Self {
        Self {
            max_frame_number: 200,
            current_frame_number: 0,
            animation_state: AnimationState::NotMoving,
            img_path: "assets/cards/glowing_frame.png",
        }
    }

    fn get_opacity(&self) -> f32 {
        let mfn = self.max_frame_number as f32;
        let cfn = self.current_frame_number as f32;
        1.0 - (0.5 - (mfn - cfn) / mfn).abs() * 2.0
    }

    fn get_rotation(&self) -> f32 {
        let mfn = self.max_frame_number as f32;
        let cfn = self.current_frame_number as f32;
        ((cfn / mfn) * 2.0 * std::f32::consts::PI).sin() * 0.05
    }
}

impl AnimationCore for FocusAnimation {
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

impl RepeatingAnimation for FocusAnimation {}

#[derive(Debug, Clone)]
pub struct Card {
    id: usize,
    img_path: &'static str,
    pub size: Size,
    pub hover_animation: HoverAnimation,
    pub remove_animation: RemoveAnimation,
    pub focus_animation: FocusAnimation
}

impl Card {

    pub fn new(id: usize, img_path: &'static str, size: Size) -> Self {
        Self {
            id: id,
            img_path: img_path,
            size: size,
            hover_animation: HoverAnimation::new(size),
            remove_animation: RemoveAnimation::new(),
            focus_animation: FocusAnimation::new()
        }
    }
}

impl GameElement for Card {

    type HigherMessage = HandMessage;
    type OwnMessage = CardMessage;

    fn convert_to_app_message(msg: CardMessage) -> AppMessage {
        Hand::convert_to_app_message(HandMessage::CardMessage(msg))
    }

    fn update_with_msg(&mut self, msg: CardMessage) {
        if self.id == msg.get_id() {
            match msg {
                CardMessage::Hovered(_) => {
                    self.hover_animation.start();
                    self.focus_animation.start();
                }
                CardMessage::Played(id) => {
                    println!("Card with id {} played!", id);
                    self.remove_animation.start();
                }
                CardMessage::NotHovered(_) => {
                    self.hover_animation.reverse();
                    self.focus_animation.reset();
                }
                CardMessage::Remove(_) => {
                    self.remove_animation.start();
                }
            }
        }
    }

    fn update_animations(&mut self) {
        self.hover_animation.next_frame();
        self.remove_animation.next_frame();
        self.focus_animation.next_frame();
    }

    fn update_size(&mut self, window_size: Size) {
        self.size = window_size;
        self.hover_animation.update_target_max_offset(self.size);
    }

    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let mut card = stack!();
        let img = image(self.img_path)
                    .content_fit(iced::ContentFit::Fill)
                    .width(self.size.width * self.hover_animation.get_size_mult() * self.remove_animation.get_contraction())
                    .height(self.size.height * self.hover_animation.get_size_mult())
                    .rotation(self.focus_animation.get_rotation())
                    .scale(0.9)
                    .opacity(self.remove_animation.get_opacity());
        card = card.push(img);

        let hover_effect = image(self.focus_animation.img_path)
                    .content_fit(iced::ContentFit::Fill)
                    .width(self.size.width * self.hover_animation.get_size_mult() * self.remove_animation.get_contraction())
                    .height(self.size.height * self.hover_animation.get_size_mult())
                    .rotation(self.focus_animation.get_rotation())
                    .scale(0.9)
                    .opacity(if self.remove_animation.get_opacity() > self.focus_animation.get_opacity() {self.focus_animation.get_opacity()} else {self.remove_animation.get_opacity()});
        card = card.push(hover_effect);

        container(MouseArea::new(card)
            .on_right_press(Hand::convert_to_app_message(HandMessage::RemoveCards))
            .on_double_click(Card::convert_to_app_message(CardMessage::Played(self.id)))
            .on_enter(Card::convert_to_app_message(CardMessage::Hovered(self.id)))
            .on_exit(Card::convert_to_app_message(CardMessage::NotHovered(self.id)))
            .interaction(Interaction::Pointer)
        )
    }

    fn view_and_move<'a>(&self, x: f32, y: f32) -> Container<'a, AppMessage> {
        
        container(pin(self.view())
            .position(
                Point::new(
                    x + ((1.0 - self.remove_animation.get_contraction()) / 2.0) * self.size.width * self.hover_animation.get_size_mult() + (self.size.width - self.size.width * self.hover_animation.get_size_mult()) / 2.0,
                    y - self.hover_animation.get_offset()
                )
            )
        )
    }
}
