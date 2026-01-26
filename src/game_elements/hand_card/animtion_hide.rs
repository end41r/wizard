use super::super::{AnimationCore, ReversableAnimation, AnimationState};

#[derive(Debug, Clone)]
pub struct HideAnimation {
    pub max_frame_number: usize,
    pub current_frame_number: usize,
    pub animation_state: AnimationState,
    pub opacity: f32,
    pub contraction: f32
}

impl HideAnimation {
    pub fn new() -> Self {
        Self {
            max_frame_number: 20,
            current_frame_number: 0,
            animation_state: AnimationState::NotMoving,
            opacity: 1.0,
            contraction: 1.0
        }
    }

    pub fn get_opacity(&self) -> f32 {
        self.opacity - self.current_frame_number as f32 / self.max_frame_number as f32
    }

    pub fn get_contraction(&self) -> f32 {
        self.contraction * (1.0 - 0.125 * self.current_frame_number as f32)
    }

    pub fn get_scale(&self) -> f32 {
        self.contraction * (1.0 - 0.125 * self.current_frame_number as f32)
    }
}

impl AnimationCore for HideAnimation {
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

impl ReversableAnimation for HideAnimation {}