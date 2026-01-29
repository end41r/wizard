use crate::animation::animation::{AnimationCore, AutoReversingAnimation, AnimationState};
use std::num::NonZero;

#[derive(Debug, Clone)]
pub struct FalsePlayedAnimation {
    pub max_frame_number: NonZero<usize>,
    pub current_frame_number: usize,
    pub animation_state: AnimationState,
}

impl FalsePlayedAnimation {
    pub fn new() -> Self {
        Self {
            max_frame_number: NonZero::new(25).unwrap(),
            current_frame_number: 0,
            animation_state: AnimationState::NotMoving,
        }
    }

    pub fn get_opacity(&self) -> f32 {
        self.current_frame_number as f32/ self.max_frame_number.get() as f32
    }
}

impl AnimationCore for FalsePlayedAnimation {
    fn max_frame_number(&mut self) -> &mut NonZero<usize> {
        &mut self.max_frame_number
    }
    fn current_frame_number(&mut self) -> &mut usize {
        &mut self.current_frame_number
    }
    fn animation_state(&mut self) -> &mut AnimationState {
        &mut self.animation_state
    }
}

impl AutoReversingAnimation for FalsePlayedAnimation {}