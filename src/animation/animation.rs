#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AnimationState {
    NotMoving,
    MovingForward,
    Reversing,
    Ended
}

pub trait AnimationCore {

    fn _mut_max_frame_number(&mut self) -> &mut usize;
    fn _mut_current_frame_number(&mut self) -> &mut usize;
    fn _mut_animation_state(&mut self) -> &mut AnimationState;

    fn start(&mut self) {
        if self.not_moving() || *self._mut_animation_state() == AnimationState::Ended {
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
    fn not_moving(&mut self) -> bool {
        *self._mut_animation_state() == AnimationState::NotMoving
    }
    fn moving_forward(&mut self) -> bool {
        *self._mut_animation_state() == AnimationState::MovingForward
    }
}

pub trait BasicAnimation: AnimationCore {
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
        *self._mut_animation_state() == AnimationState::Ended
    }
}

pub trait RepeatingBasicAnimation: AnimationCore {
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

pub trait ReversableAnimation: AnimationCore {
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
                    *self._mut_animation_state() = AnimationState::Ended;
                }
            }
            _ => {}
        }
    }
    fn reversing(&mut self) -> bool {
        *self._mut_animation_state() == AnimationState::Reversing
    }
}

pub trait AutoReversingAnimation: AnimationCore {
    fn next_frame(&mut self) {
        match self._mut_animation_state() {
            AnimationState::MovingForward => {
                if *self._mut_current_frame_number() < *self._mut_max_frame_number() {
                    *self._mut_current_frame_number() += 1;
                } else {
                    *self._mut_animation_state() = AnimationState::Reversing;
                }
            }
            AnimationState::Reversing => {
                if *self._mut_current_frame_number() > 0 {
                    *self._mut_current_frame_number() -= 1;
                } else {
                    self.reset();
                }
            }
            _ => {}
        }
    }
    fn reversing(&mut self) -> bool {
        *self._mut_animation_state() == AnimationState::Reversing
    }
}

pub trait RepeatingAutoReversingAnimation: AnimationCore {
    fn next_frame(&mut self) {
        match self._mut_animation_state() {
            AnimationState::MovingForward => {
                if *self._mut_current_frame_number() < *self._mut_max_frame_number() {
                    *self._mut_current_frame_number() += 1;
                } else {
                    *self._mut_animation_state() = AnimationState::Reversing;
                }
            }
            AnimationState::Reversing => {
                if *self._mut_current_frame_number() > 0 {
                    *self._mut_current_frame_number() -= 1;
                } else {
                    *self._mut_animation_state() = AnimationState::MovingForward;
                }
            }
            _ => {}
        }
    }
    fn reversing(&mut self) -> bool {
        *self._mut_animation_state() == AnimationState::Reversing
    }
}