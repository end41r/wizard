pub mod hand;
pub mod hand_card;

use std::num::NonZero;
use iced::{Size, widget::Container};
use crate::client::AppMessage;

pub trait GameElement {
    type OwnMessage;
    fn convert_to_app_message(msg: Self::OwnMessage) -> AppMessage;
    /// Convey the msg to lower GameElements asap (if they exist) before doing anything else.
    fn update_with_msg(&mut self, msg: Self::OwnMessage);
    /// Call this every AnimationTick.
    /// First call other update_animations then animation tickers.
    fn update_animations(&mut self);
    /// Call this every window resize.
    /// If you create new game elements within the update function make sure to call this function
    /// to give them their correct size!
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

/// An AnimationEndSensor executes a caode block when an animation ends.
/// Start the sensor with fn start and execute the code with fn check.
#[derive(Debug)]
pub struct AnimationEndSensor<C> {
    animation_length: NonZero<usize>,
    tick: usize,
    state: AnimtionTickerState,
    content: Option<C>
}

impl<C> AnimationEndSensor<C> {
    fn new(duration: NonZero<usize>) -> Self {
        Self {
            animation_length: duration,
            tick: 0,
            state: AnimtionTickerState::Inactive,
            content: None
        }
    }
    fn content(&self) -> Option<&C> {
        self.content.as_ref()
    }
    fn active(&self) -> bool {
        match self.state {
            AnimtionTickerState::Active => true,
            AnimtionTickerState::Inactive => false
        }
    }
    fn last_tick_reached(&self) -> bool {
        self.tick == self.animation_length.get()
    }
    fn start(&mut self, content: Option<C>) {
        if self.state == AnimtionTickerState::Inactive {
            self.state = AnimtionTickerState::Active
        }
        self.content = content;
    }
    fn reset(&mut self) {
        self.state = AnimtionTickerState::Inactive;
        self.tick = 0;
    }
    /// Use this every time update_animations from the GameElement trait is called.
    /// 
    /// This function executes the action when the last tick is reached.
    /// 
    /// This function returns true when the last tick is reached.
    /// This property is useful when facing borrowing issues within the action.
    fn check<A>(&mut self, action: A) -> bool where A: FnOnce(&mut Self) {
        if self.state == AnimtionTickerState::Active {
            if self.last_tick_reached() {
                action(self);
                self.reset();
                return true;
            } else {
                self.tick += 1;
            };
        }
        false
    }
}

/// An AnimationStarter is used for starting multiple animations with a delay.
/// It allows you to execute a code block when the last animation has started (NOT ended).
#[derive(Debug)]
pub struct AnimationStarter<C> {
    animation_delay: NonZero<usize>,
    tick: usize,
    times: usize,
    state: AnimtionTickerState,
    content: Option<C>
}

impl<C> AnimationStarter<C> {
    fn new(delay: NonZero<usize>) -> Self {
        Self {
            animation_delay: delay,
            tick: 0,
            times: 0,  // Will be set in fn start() where it first matters
            state: AnimtionTickerState::Inactive,
            content: None
        }
    }
    fn content(&self) -> Option<&C> {
        self.content.as_ref()
    }
    fn active(&self) -> bool {
        match self.state {
            AnimtionTickerState::Active => true,
            AnimtionTickerState::Inactive => false
        }
    }
    fn last_tick_reached(& self) -> bool {
        self.tick == self.times * self.animation_delay.get()
    }
    fn start(&mut self, content: Option<C>, times: usize) {
        if self.state == AnimtionTickerState::Inactive {
            self.state = AnimtionTickerState::Active
        }
        self.times = if times == 0 {times} else {times - 1};
        self.content = content;
    }
    fn reset(&mut self) {
        self.state = AnimtionTickerState::Inactive;
        self.tick = 0;
    }
    /// Use this every time update_animations from the GameElement trait is called.
    /// 
    /// This function executes the action used for starting an animation
    /// everytime the delay period has passed.
    /// 
    /// This function returns true when the last tick is reached
    /// and all animations are started (NOT ended).
    /// You can use this property with an if-statement then to immediately execute an action.
    fn check<A>(&mut self, action: A) -> bool where A: FnOnce(&mut Self) {
        if self.state == AnimtionTickerState::Active {
            if self.tick % self.animation_delay == 0 {
                action(self);
            }
            if self.last_tick_reached() {
                self.reset();
                return true;
            } else {
                self.tick += 1;
            }
        };
        false
    }
    fn cycle(&self) -> usize {
        (self.tick - (self.tick % self.animation_delay)) / self.animation_delay
    }
}