use super::super::{AnimationCore, RepeatingAutoReversingAnimation, AnimationState};

#[derive(Debug, Clone)]
pub struct PlayableAnimation {
    pub max_frame_number: usize,
    pub current_frame_number: usize,
    pub animation_state: AnimationState,
}

impl PlayableAnimation {
    pub fn new() -> Self {
        Self {
            max_frame_number: 150,
            current_frame_number: 0,
            animation_state: AnimationState::NotMoving,
        }
    }

    pub fn get_opacity(&self) -> f32 {
        let mfn: f32 = self.max_frame_number as f32;
        let cfn = self.current_frame_number as f32;
        (cfn / mfn) * 0.3 + 0.7
    }
}

impl AnimationCore for PlayableAnimation {
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

impl RepeatingAutoReversingAnimation for PlayableAnimation {}