use super::super::{AnimationCore, RepeatingAnimation, AnimationState};

#[derive(Debug, Clone)]
pub struct HoverFocusAnimation {
    pub max_frame_number: usize,
    pub current_frame_number: usize,
    pub animation_state: AnimationState,
    pub img_path: &'static str,
}

impl HoverFocusAnimation {
    pub fn new() -> Self {
        Self {
            max_frame_number: 200,
            current_frame_number: 0,
            animation_state: AnimationState::NotMoving,
            img_path: "assets/cards/frame_yellow.png",
        }
    }

    pub fn get_opacity(&self) -> f32 {
        let mfn = self.max_frame_number as f32;
        let cfn = self.current_frame_number as f32;
        1.0 - (0.5 - (mfn - cfn) / mfn).abs() * 2.0
    }

    pub fn get_rotation(&self) -> f32 {
        let mfn = self.max_frame_number as f32;
        let cfn = self.current_frame_number as f32;
        ((cfn / mfn) * 2.0 * std::f32::consts::PI).sin() * 0.05
    }
}

impl AnimationCore for HoverFocusAnimation {
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

impl RepeatingAnimation for HoverFocusAnimation {}