use iced::Size;
use super::super::{AnimationCore, ReversableAnimation, AnimationState};

#[derive(Debug, Clone)]
pub struct HoverAnimation {
    pub max_frame_number: usize,
    pub current_frame_number: usize,
    pub animation_state: AnimationState,

    pub max_offset: f32
}

impl HoverAnimation {
    pub fn new(size: Size) -> Self {
        Self {
            max_frame_number: 5,
            current_frame_number: 0,
            animation_state: AnimationState::NotMoving,
            max_offset: size.height * 0.15,
        }
    }
    pub fn update_target_max_offset(&mut self, size: Size) {
        self.max_offset = size.height * 0.15;
    }
    pub fn get_offset(&self) -> f32 {
        self.max_offset * 0.2 * self.current_frame_number as f32
    }
    pub fn get_expansion(&self) -> f32 {
        1.0 + self.current_frame_number as f32 * 0.02
    }
}

impl AnimationCore for HoverAnimation {
    fn _mut_max_frame_number(&mut self) -> &mut usize {
        &mut self.max_frame_number
    }
    fn _mut_current_frame_number(&mut self) -> &mut usize {
        &mut self.current_frame_number
    }
    fn _mut_animation_state(&mut self) -> &mut AnimationState {
        &mut self.animation_state
    }
} 

impl ReversableAnimation for HoverAnimation {}
