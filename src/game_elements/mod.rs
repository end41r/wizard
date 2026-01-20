pub mod hand;
pub mod hand_card;

use iced::{Size, widget::Container};
use crate::client::AppMessage;

pub trait GameElement {
    type HigherMessage;
    type OwnMessage;
    fn convert_to_app_message(msg: Self::OwnMessage) -> AppMessage;
    /// Convey the msg to lower GameElements asap (if they exist) before doing anything else.
    fn update_with_msg(&mut self, msg: Self::OwnMessage);
    /// Call this every AnimationTick.
    /// First call other update_animations then animation tickers.
    fn update_animations(&mut self);
    /// Call this every window resize.
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
    fn active(&mut self) -> bool{
        match *self._mut_animation_state() {
            AnimationState::NotMoving => false,
            AnimationState::Ended => false,
            _ => true
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AnimationState {
    NotMoving,
    MovingForward,
    Reversing,
    Ended
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
                    *self._mut_animation_state() = AnimationState::Ended;
                }
            }
            _ => {}
        }
    }

    fn ended(&mut self) -> bool {
        match *self._mut_animation_state() {
            AnimationState::Ended => true,
            _ => false
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

    fn start_from_reverse(&mut self) {
        *self._mut_animation_state() = AnimationState::Reversing;
        *self._mut_current_frame_number() = *self._mut_max_frame_number()
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

#[derive(PartialEq, Debug)]
enum AnimtionTickerState {
    Active,
    Inactive,
}

/// This struct is designed for 2 use cases:
/// 1.  Starting multiple animations with a delay:
///     interval = delay, times = amount of animations
/// 2.  Detecting if an animation has ended:
///     interval = animation length, times = 1
#[derive(Debug)]
pub struct AnimationTicker {
    interval: usize,
    tick: usize,
    times: usize,
    state: AnimtionTickerState
}

impl AnimationTicker {
    fn new(interval: usize, times: usize) -> Self {
        Self {
            interval: interval,
            tick: 0,
            times: times,
            state: AnimtionTickerState::Inactive
        }
    }
    fn start(&mut self) {
        if self.state == AnimtionTickerState::Inactive {
            self.state = AnimtionTickerState::Active
        }
    }
    /// Use this every time update_animations from the GameElement trait is called.
    /// 
    /// Retuns true if the current AnimationTick for an action is reached
    /// 
    /// Example for interval = 3, times = 4 (x = true, o = false):
    /// o o x o o x o o x o o x
    fn check(&mut self) -> bool {
        if self.state == AnimtionTickerState::Active {
            if self.tick == self.times * self.interval {  // Last tick reached
                self.state = AnimtionTickerState::Inactive;
                self.tick = 0;
                return true
            };
            
            self.tick += 1;

            if self.tick - 1 == 0 {
                return false  // Not on first tick
            } else {
                return (self.tick - 1) % self.interval == 0
            }
        }
        false
    }
}