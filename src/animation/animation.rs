/// For animations you first need to create a struct (e.g. MyAnimation) with 3 things:
///     - max_frame_number: NonZero<usize>
///     - current_frame_number: usize
///     - animation_state: AnimationState
/// Then you implement AnimationCore with its 3 getters.
/// Then implement your fitting animation type (e.g. impl BasicAnimation for MyAnimation{})
/// Add a new(...) -> Self function to impl MyAnimation{}
/// Now add functions that calculate what you want using the progress function
/// which gives you the relation between the current and the max frame number
/// (ranging from 0.0 tp 1.0), e.g.:
/// get_opacity(&self) -> f32 {self.progress(Easing::EaseInCubic)}
use crate::animation::{
    ease_in_out_sine, ease_in_sine, ease_out_bounce, ease_out_elastic, ease_out_sine,
};
use derive_more::{Deref, DerefMut};
use std::num::NonZero;

use super::{ease_in_cubic, ease_in_out_cubic, ease_out_cubic, AnimationState, Easing};

// AI-Usage: Claude.ai for learning how to make a trait require another trait.
//           (Now this is not the case here anymore but it used to be)

#[derive(Debug, Clone)]
pub struct Animation {
    max_frame_number: NonZero<usize>,
    current_frame_number: usize,
    animation_state: AnimationState
}

impl Animation {
    pub fn new_core(duration: NonZero<usize>) -> Self {
        Self {
            max_frame_number: duration,
            current_frame_number: 0,
            animation_state: AnimationState::NotMoving
        }
    }
    pub fn start(&mut self) {
        if self.not_moving() || self.animation_state == AnimationState::Ended {
            self.animation_state = AnimationState::MovingForward;
        }
    }
    pub fn interrupt(&mut self) {
        self.animation_state = AnimationState::NotMoving;
    }
    pub fn reset(&mut self) {
        self.current_frame_number = 0;
        self.animation_state = AnimationState::NotMoving;
    }
    /// This function represents the progress of the animation ranging from 0.0 to 1.0.
    /// Choose an easing type to manipulate the look of the animation to your liking.
    pub fn progress(&self, curve: Easing) -> f32 {
        let progress: f32 =
            self.current_frame_number as f32 / self.max_frame_number.get() as f32;
        match curve {
            Easing::Linear => progress,
            Easing::InCubic => ease_in_cubic(progress),
            Easing::OutCubic => ease_out_cubic(progress),
            Easing::InOutCubic => ease_in_out_cubic(progress),
            Easing::InSine => ease_in_sine(progress),
            Easing::OutSine => ease_out_sine(progress),
            Easing::InOutSine => ease_in_out_sine(progress),
            Easing::OutElastic => ease_out_elastic(progress),
            Easing::OutBounce => ease_out_bounce(progress),
        }
    }
    pub fn not_moving(&self) -> bool {
        self.animation_state == AnimationState::NotMoving
    }
    pub fn moving_forward(&self) -> bool {
        self.animation_state == AnimationState::MovingForward
    }
}


// AI-Usage: Claude.ai to learn how to use a macro and partially generate the code regarding
//           the macro.
macro_rules! new_core {
    ($name:ident) => {
        impl $name {
            // This is marked as not used because CircularAnimation is as of now not used.
            pub fn new(duration: NonZero<usize>) -> Self {
                Self(Animation::new_core(duration))
            }
        }
    };
}


new_core!(BasicAnimation);
new_core!(CircularAnimation);
new_core!(ReversableBasicAnimation);
new_core!(AutoReversingAnimation);
new_core!(CircularAutoReversingAnimation);

macro_rules! new_basic {
    ($name:ident) => {
        impl $name {
            // This is marked as not used because CircularAnimation is as of now not used.
            pub fn new(duration: NonZero<usize>) -> Self {
                Self(BasicAnimation::new(duration))
            }
        }
    };
}
pub (crate) use new_basic;

macro_rules! new_circular {
    ($name:ident) => {
        impl $name {
            // This is marked as not used because CircularAnimation is as of now not used.
            pub fn new(duration: NonZero<usize>) -> Self {
                Self(CircularAnimation::new(duration))
            }
        }
    };
}
pub (crate) use new_circular;

macro_rules! new_reversable_basic {
    ($name:ident) => {
        impl $name {
            // This is marked as not used because CircularAnimation is as of now not used.
            pub fn new(duration: NonZero<usize>) -> Self {
                Self(ReversableBasicAnimation::new(duration))
            }
        }
    };
}
pub (crate) use new_reversable_basic;

macro_rules! new_auto_reversing {
    ($name:ident) => {
        impl $name {
            // This is marked as not used because CircularAnimation is as of now not used.
            pub fn new(duration: NonZero<usize>) -> Self {
                Self(AutoReversingAnimation::new(duration))
            }
        }
    };
}
pub (crate) use new_auto_reversing;

macro_rules! new_circular_auto_reversing {
    ($name:ident) => {
        impl $name {
            // This is marked as not used because CircularAnimation is as of now not used.
            pub fn new(duration: NonZero<usize>) -> Self {
                Self(CircularAutoReversingAnimation::new(duration))
            }
        }
    };
}
pub (crate) use new_circular_auto_reversing;

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct BasicAnimation(Animation);
impl BasicAnimation {
    pub fn next_frame(&mut self) {
        match self.animation_state {
            AnimationState::MovingForward => {
                if self.current_frame_number < self.max_frame_number.get() {
                    self.current_frame_number += 1;
                } else {
                    self.animation_state = AnimationState::Ended;
                }
            }
            _ => {}
        }
    }
    pub fn ended(&self) -> bool {
        self.animation_state == AnimationState::Ended
    }
}

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct CircularAnimation(Animation);
impl CircularAnimation{
    pub fn next_frame(&mut self) {
        match self.animation_state {
            AnimationState::MovingForward => {
                self.current_frame_number =
                // The 1 is there for going to the next frame number.
                    (self.current_frame_number + 1) % self.max_frame_number.get();
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct ReversableBasicAnimation(Animation);
impl ReversableBasicAnimation{
    pub fn reverse(&mut self) {
        self.animation_state = AnimationState::Reversing;
    }
    pub fn start_from_reverse(&mut self) {
        self.animation_state = AnimationState::Reversing;
        self.current_frame_number = self.max_frame_number.get()
    }
    pub fn next_frame(&mut self) {
        match self.animation_state {
            AnimationState::MovingForward => {
                if self.current_frame_number < self.max_frame_number.get() {
                    self.current_frame_number += 1;
                } else {
                    self.animation_state = AnimationState::NotMoving;
                }
            }
            AnimationState::Reversing => {
                if self.current_frame_number > 0 {
                    self.current_frame_number -= 1;
                } else {
                    self.animation_state = AnimationState::Ended;
                }
            }
            _ => {}
        }
    }
    pub fn reversing(&self) -> bool {
        self.animation_state == AnimationState::Reversing
    }
}

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct AutoReversingAnimation(Animation);
impl AutoReversingAnimation {
    pub fn next_frame(&mut self) {
        match self.animation_state {
            AnimationState::MovingForward => {
                if self.current_frame_number < self.max_frame_number.get() {
                    self.current_frame_number += 1;
                } else {
                    self.animation_state = AnimationState::Reversing;
                }
            }
            AnimationState::Reversing => {
                if self.current_frame_number > 0 {
                    self.current_frame_number -= 1;
                } else {
                    self.reset();
                }
            }
            _ => {}
        }
    }
    pub fn reversing(&self) -> bool {
        self.animation_state == AnimationState::Reversing
    }
}

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct CircularAutoReversingAnimation(Animation);
impl CircularAutoReversingAnimation{
    pub fn next_frame(&mut self) {
        match self.animation_state {
            AnimationState::MovingForward => {
                if self.current_frame_number < self.max_frame_number.get() {
                    self.current_frame_number += 1;
                } else {
                    self.animation_state = AnimationState::Reversing;
                }
            }
            AnimationState::Reversing => {
                if self.current_frame_number > 0 {
                    self.current_frame_number -= 1;
                } else {
                    self.animation_state = AnimationState::MovingForward;
                }
            }
            _ => {}
        }
    }
    pub fn reversing(&self) -> bool {
        self.animation_state == AnimationState::Reversing
    }
}
