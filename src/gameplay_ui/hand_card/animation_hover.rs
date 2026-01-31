use crate::animation::animation::{AnimationCore, AnimationState, ReversableBasicAnimation};
use std::num::NonZero;

#[derive(Debug, Clone)]
pub struct HoverAnimation {
    pub max_frame_number: NonZero<usize>,
    pub current_frame_number: usize,
    pub animation_state: AnimationState,

    pub max_offset: f32,
}

impl HoverAnimation {
    pub fn new(card_height: f32) -> Self {
        Self {
            max_frame_number: NonZero::new(5).unwrap(),
            current_frame_number: 0,
            animation_state: AnimationState::NotMoving,
            max_offset: card_height * 0.15,
        }
    }
    pub fn update_max_offset(&mut self, card_height: f32) {
        self.max_offset = card_height * 0.15;
    }
    pub fn get_offset(&self) -> f32 {
        self.max_offset * self.current_frame_number as f32 / self.max_frame_number.get() as f32
    }
    // The factor 0.02 partially determines the hand width in hand::Hand::width_overflow_one_side.
    pub fn get_expansion(&self) -> f32 {
        1.0 + self.current_frame_number as f32 * 0.02
    }
}

impl AnimationCore for HoverAnimation {
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

impl ReversableBasicAnimation for HoverAnimation {}
