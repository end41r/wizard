use super::super::{AnimationCore, AutoReversingAnimation, AnimationState};

#[derive(Debug, Clone)]
pub struct FalsePlayedAnimation {
    pub max_frame_number: usize,
    pub current_frame_number: usize,
    pub animation_state: AnimationState,
}

impl FalsePlayedAnimation {
    pub fn new() -> Self {
        Self {
            max_frame_number: 25,
            current_frame_number: 0,
            animation_state: AnimationState::NotMoving,
        }
    }

    pub fn get_opacity(&self) -> f32 {
        let mfn: f32 = self.max_frame_number as f32;
        let cfn: f32 = self.current_frame_number as f32;
        cfn / mfn
    }
}

impl AnimationCore for FalsePlayedAnimation {
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

impl AutoReversingAnimation for FalsePlayedAnimation {}