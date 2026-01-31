use crate::animation::animation::{AutoReversingAnimation};
use crate::animation::Easing;
use std::num::NonZero;
use derive_more::{Deref, DerefMut};

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct FalsePlayedAnimation(AutoReversingAnimation);

impl FalsePlayedAnimation {
    pub fn new(duration: NonZero<usize>) -> Self {
        Self(AutoReversingAnimation::new(duration))
    }
    pub fn get_opacity(&self) -> f32 {
        self.progress(Easing::InOutSine)
    }
}
