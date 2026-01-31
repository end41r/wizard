use crate::animation::animation::{AnimationCore, AnimationState, CircularAutoReversingAnimation};
use std::num::NonZero;

#[derive(Debug, Clone)]
pub struct PlayableAnimation {
    pub max_frame_number: NonZero<usize>,
    pub current_frame_number: usize,
    pub animation_state: AnimationState,
}

impl PlayableAnimation {
    pub fn new() -> Self {
        Self {
            max_frame_number: NonZero::new(150).unwrap(),
            current_frame_number: 0,
            animation_state: AnimationState::NotMoving,
        }
    }

    pub fn get_opacity(&self) -> f32 {
        // This only affects the last 30% of the opacity.
        self.progress() * 0.3 + 0.7
    }
}

impl AnimationCore for PlayableAnimation {
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

impl CircularAutoReversingAnimation for PlayableAnimation {}
