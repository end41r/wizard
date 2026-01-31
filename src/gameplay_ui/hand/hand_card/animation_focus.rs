use crate::animation::{CircularAutoReversingAnimation, Easing};
use derive_more::{Deref, DerefMut};
use std::num::NonZero;

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct FocusAnimation(CircularAutoReversingAnimation);

impl FocusAnimation {
    pub fn new(duration: NonZero<usize>) -> Self {
        Self(CircularAutoReversingAnimation::new(duration))
    }
    pub fn get_opacity(&self) -> f32 {
        self.progress(Easing::InOutCubic)
    }
}
