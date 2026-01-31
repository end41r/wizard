use crate::animation::animation::{AnimationCore, AnimationState, ReversableBasicAnimation};
use std::num::NonZero;

#[derive(Debug, Clone)]
pub struct HideAnimation {
    pub max_frame_number: NonZero<usize>,
    pub current_frame_number: usize,
    pub animation_state: AnimationState,
}

impl HideAnimation {
    pub fn new() -> Self {
        Self {
            // Needs to be as high as the max_frame_number of the PlayAnimation.
            max_frame_number: NonZero::new(12).unwrap(),
            current_frame_number: 0,
            animation_state: AnimationState::NotMoving,
        }
    }

    pub fn get_opacity(&self) -> f32 {
        1.0 - self.progress()
    }

    pub fn get_contraction(&self) -> f32 {
        1.0 - self.progress()
    }

    pub fn get_scale(&self) -> f32 {
        1.0 - self.progress()
    }
}

impl AnimationCore for HideAnimation {
    fn max_frame_number(&self) -> NonZero<usize> {
        self.max_frame_number
    }
    fn current_frame_number(&self) -> usize {
        self.current_frame_number
    }
    fn animation_state(&self) -> AnimationState {
        self.animation_state
    }
    fn _mut_max_frame_number(&mut self) -> &mut NonZero<usize> {
        &mut self.max_frame_number
    }
    fn _mut_current_frame_number(&mut self) -> &mut usize {
        &mut self.current_frame_number
    }
    fn _mut_animation_state(&mut self) -> &mut AnimationState {
        &mut self.animation_state
    }
}

impl ReversableBasicAnimation for HideAnimation {}
