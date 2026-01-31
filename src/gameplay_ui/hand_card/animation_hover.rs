use crate::animation::animation::{AnimationCore, ReversableBasicAnimation};
use crate::animation::{AnimationState, Easing};
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
        // The factor 0.15 determines the height of the hand.
        self.max_offset = card_height * 0.15;
    }
    pub fn get_offset(&self) -> f32 {
        self.max_offset * self.progress(Easing::Linear)
    }
    // The factor 0.1 partially determines the hand width in hand::Hand::width_overflow_one_side.
    pub fn get_expansion(&self) -> f32 {
        1.0 + self.progress(Easing::Linear) * 0.1
    }
}

impl AnimationCore for HoverAnimation {
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

impl ReversableBasicAnimation for HoverAnimation {}
