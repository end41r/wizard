use crate::animation::{CircularAutoReversingAnimation, Easing};
use derive_more::{Deref, DerefMut};

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct FocusAnimation(CircularAutoReversingAnimation);

impl FocusAnimation {
    pub fn new(duration: usize) -> Self {
        Self(CircularAutoReversingAnimation::new(duration))
    }
    pub fn get_opacity(&self) -> f32 {
        self.progress(Easing::InOutCubic)
    }
}
