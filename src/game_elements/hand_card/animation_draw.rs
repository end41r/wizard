use super::super::{AnimationCore, BasicAnimation, AnimationState};
use super::f32_min_2;

#[derive(Debug, Clone)]
pub struct DrawAnimation {
    pub max_frame_number: usize,
    pub current_frame_number: usize,
    pub animation_state: AnimationState,
}

impl DrawAnimation {
    pub fn new() -> Self {
        Self {
            max_frame_number: 10,
            current_frame_number: 0,
            animation_state: AnimationState::NotMoving,
        }
    }

    pub fn get_opacity(&self) -> f32 {
        self.current_frame_number as f32 * self.max_frame_number as f32
    }

    pub fn get_contraction(&self) -> f32 {
        self.current_frame_number as f32 / self.max_frame_number as f32
    }

    pub fn get_scale(&self) -> f32 {
        f32_min_2(self.current_frame_number as f32 / self.max_frame_number as f32 + 0.5, 1.0)
    }
}

impl AnimationCore for DrawAnimation {
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

impl BasicAnimation for DrawAnimation {}