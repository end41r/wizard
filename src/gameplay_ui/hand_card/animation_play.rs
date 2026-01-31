use crate::animation::animation::{AnimationCore, AnimationState, BasicAnimation};
use std::num::NonZero;

#[derive(Debug, Clone)]
pub struct PlayAnimation {
    pub max_frame_number: NonZero<usize>,
    pub current_frame_number: usize,
    pub animation_state: AnimationState,
}

impl PlayAnimation {
    pub fn new() -> Self {
        Self {
            // Needs to be as high as the max_frame_number of the HideAnimation.
            max_frame_number: NonZero::new(12).unwrap(),
            current_frame_number: 0,
            animation_state: AnimationState::NotMoving,
        }
    }

    pub fn get_opacity(&self) -> f32 {
        1.0 - self.current_frame_number as f32 / self.max_frame_number.get() as f32
    }

    pub fn get_contraction(&self) -> f32 {
        1.0 - self.current_frame_number as f32 / self.max_frame_number.get() as f32
    }
}

impl AnimationCore for PlayAnimation {
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

impl BasicAnimation for PlayAnimation {}
