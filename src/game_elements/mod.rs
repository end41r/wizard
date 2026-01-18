pub mod hand;
pub mod hand_card;

use iced::{Size, widget::Container};
use crate::client::AppMessage;

pub trait GameElement {
    type HigherMessage;
    type OwnMessage;
    fn convert_msg(msg: Self::HigherMessage) -> Self::OwnMessage;
    fn convert_to_app_message(msg: Self::OwnMessage) -> AppMessage;
    fn update_with_msg(&mut self, msg: Self::OwnMessage);
    fn update_animations(&mut self);
    fn update_size(&mut self, window_size: Size);
    fn view<'a>(&self) -> Container<'a, AppMessage>;
    fn view_and_move<'a>(&self, x: f32, y: f32) -> Container<'a, AppMessage>;
}

pub trait AnimationCore {

    fn _mut_max_frame_number(&mut self) -> &mut usize;
    fn _mut_current_frame_number(&mut self) -> &mut usize;
    fn _mut_animation_state(&mut self) -> &mut AnimationState;

    fn interrupt(&mut self) {
        *self._mut_animation_state() = AnimationState::NotMoving;
    }
    fn reset(&mut self) {
        *self._mut_current_frame_number() = 0;
        *self._mut_animation_state() = AnimationState::NotMoving;
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AnimationState {
    NotMoving,
    MovingForward,
    Reversing,
}

pub trait BasicAnimation: AnimationCore {

    fn start(&mut self) {
       *self._mut_animation_state() = AnimationState::MovingForward;
    }

    fn next_frame(&mut self) {
        match self._mut_animation_state() {
            AnimationState::MovingForward => {
                if *self._mut_current_frame_number() < *self._mut_max_frame_number() {
                    *self._mut_current_frame_number() += 1;
                } else {
                    *self._mut_animation_state() = AnimationState::NotMoving;
                }
            }
            _ => {}
        }
    }
}

pub trait ReversableAnimation: AnimationCore {

    fn start(&mut self) {
       *self._mut_animation_state() = AnimationState::MovingForward;
    }

    fn reverse(&mut self) {
        *self._mut_animation_state() = AnimationState::Reversing;
    }

    fn next_frame(&mut self) {
        match self._mut_animation_state() {
            AnimationState::MovingForward => {
                if *self._mut_current_frame_number() < *self._mut_max_frame_number() {
                    *self._mut_current_frame_number() += 1;
                } else {
                    *self._mut_animation_state() = AnimationState::NotMoving;
                }
            }
            AnimationState::Reversing => {
                if *self._mut_current_frame_number() > 0 {
                    *self._mut_current_frame_number() -= 1;
                } else {
                    *self._mut_animation_state() = AnimationState::NotMoving;
                }
            }
            _ => {}
        }
    }
}

pub trait RepeatingAnimation: AnimationCore {

    fn start(&mut self) {
       *self._mut_animation_state() = AnimationState::MovingForward;
    }

    fn next_frame(&mut self) {
        match self._mut_animation_state() {
            AnimationState::MovingForward => {
                *self._mut_current_frame_number()
                    = (*self._mut_current_frame_number() + 1) % *self._mut_max_frame_number();
            }
            _ => {}
        }
    }
}