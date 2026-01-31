use crate::animation::{
    ease_in_out_sine, ease_in_sine, ease_out_bounce, ease_out_elastic, ease_out_sine,
};
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
use std::num::NonZero;

use super::{ease_in_cubic, ease_in_out_cubic, ease_out_cubic, AnimationState, Easing};

pub trait AnimationCore {
    fn max_frame_number(&self) -> NonZero<usize>;
    fn current_frame_number(&self) -> usize;
    fn animation_state(&self) -> AnimationState;
    fn _mut_max_frame_number(&mut self) -> &mut NonZero<usize>;
    fn _mut_current_frame_number(&mut self) -> &mut usize;
    fn _mut_animation_state(&mut self) -> &mut AnimationState;

    fn start(&mut self) {
        if self.not_moving() || self.animation_state() == AnimationState::Ended {
            *self._mut_animation_state() = AnimationState::MovingForward;
        }
    }
    fn interrupt(&mut self) {
        *self._mut_animation_state() = AnimationState::NotMoving;
    }
    fn reset(&mut self) {
        *self._mut_current_frame_number() = 0;
        *self._mut_animation_state() = AnimationState::NotMoving;
    }
    /// This function represents the progress of the animation ranging from 0.0 to 1.0.
    /// Choose an easing type to manipulate the look of the animation to your liking.
    fn progress(&self, curve: Easing) -> f32 {
        let progress: f32 =
            self.current_frame_number() as f32 / self.max_frame_number().get() as f32;
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
    fn not_moving(&self) -> bool {
        self.animation_state() == AnimationState::NotMoving
    }
    fn moving_forward(&self) -> bool {
        self.animation_state() == AnimationState::MovingForward
    }
}

// AI-Usage: Claude.ai for learning how to make a trait require another trait.
pub trait BasicAnimation: AnimationCore {
    fn next_frame(&mut self) {
        match self.animation_state() {
            AnimationState::MovingForward => {
                if self.current_frame_number() < self.max_frame_number().get() {
                    *self._mut_current_frame_number() += 1;
                } else {
                    *self._mut_animation_state() = AnimationState::Ended;
                }
            }
            _ => {}
        }
    }
    fn ended(&self) -> bool {
        self.animation_state() == AnimationState::Ended
    }
}

pub trait CircularAnimation: AnimationCore {
    fn next_frame(&mut self) {
        match self.animation_state() {
            AnimationState::MovingForward => {
                *self._mut_current_frame_number() =
                // The 1 is there for going to the next frame number.
                    (self.current_frame_number() + 1) % self.max_frame_number().get();
            }
            _ => {}
        }
    }
}

pub trait ReversableBasicAnimation: AnimationCore {
    fn reverse(&mut self) {
        *self._mut_animation_state() = AnimationState::Reversing;
    }
    fn start_from_reverse(&mut self) {
        *self._mut_animation_state() = AnimationState::Reversing;
        *self._mut_current_frame_number() = self.max_frame_number().get()
    }
    fn next_frame(&mut self) {
        match self.animation_state() {
            AnimationState::MovingForward => {
                if self.current_frame_number() < self.max_frame_number().get() {
                    *self._mut_current_frame_number() += 1;
                } else {
                    *self._mut_animation_state() = AnimationState::NotMoving;
                }
            }
            AnimationState::Reversing => {
                if self.current_frame_number() > 0 {
                    *self._mut_current_frame_number() -= 1;
                } else {
                    *self._mut_animation_state() = AnimationState::Ended;
                }
            }
            _ => {}
        }
    }
    fn reversing(&self) -> bool {
        self.animation_state() == AnimationState::Reversing
    }
}

pub trait AutoReversingAnimation: AnimationCore {
    fn next_frame(&mut self) {
        match self.animation_state() {
            AnimationState::MovingForward => {
                if self.current_frame_number() < self.max_frame_number().get() {
                    *self._mut_current_frame_number() += 1;
                } else {
                    *self._mut_animation_state() = AnimationState::Reversing;
                }
            }
            AnimationState::Reversing => {
                if self.current_frame_number() > 0 {
                    *self._mut_current_frame_number() -= 1;
                } else {
                    self.reset();
                }
            }
            _ => {}
        }
    }
    fn reversing(&self) -> bool {
        self.animation_state() == AnimationState::Reversing
    }
}

pub trait CircularAutoReversingAnimation: AnimationCore {
    fn next_frame(&mut self) {
        match self.animation_state() {
            AnimationState::MovingForward => {
                if self.current_frame_number() < self.max_frame_number().get() {
                    *self._mut_current_frame_number() += 1;
                } else {
                    *self._mut_animation_state() = AnimationState::Reversing;
                }
            }
            AnimationState::Reversing => {
                if self.current_frame_number() > 0 {
                    *self._mut_current_frame_number() -= 1;
                } else {
                    *self._mut_animation_state() = AnimationState::MovingForward;
                }
            }
            _ => {}
        }
    }
    fn reversing(&self) -> bool {
        self.animation_state() == AnimationState::Reversing
    }
}
