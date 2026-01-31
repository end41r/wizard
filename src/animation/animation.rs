/// For animations you first need to create a struct (e.g. MyAnimation) with 3 things:
///     - max_frame_number: NonZero<usize>
///     - current_frame_number: usize
///     - animation_state: AnimationState
/// Then you implement AnimationCore with its 3 getters.
/// Then implement your fitting animation type (e.g. impl BasicAnimation for MyAnimation{})
/// Add a new(...) -> Self function to impl MyAnimation{}
/// Now add functions that calculate what you want to animate using the relation of
/// the current and max frame number, e.g.:
/// get_offset(&self) -> f32 {self.current_frame_number as f32 /self.max_frame_number.get() as f32}
use std::num::NonZero;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AnimationState {
    NotMoving,
    MovingForward,
    Reversing,
    Ended,
}

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
