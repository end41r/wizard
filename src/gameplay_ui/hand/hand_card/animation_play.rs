use crate::animation::{BasicAnimation, Easing};
use derive_more::{Deref, DerefMut};
use std::num::NonZero;

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct PlayAnimation(BasicAnimation);

impl PlayAnimation {
    pub fn new(duration: NonZero<usize>) -> Self {
        Self(BasicAnimation::new(duration))
    }
    pub fn get_opacity(&self) -> f32 {
        1.0 - self.progress(Easing::InCubic)
    }
    pub fn get_contraction(&self) -> f32 {
        1.0 - self.progress(Easing::InSine)
    }
}
