use crate::animation::{CircularAutoReversingAnimation, Easing};
use derive_more::{Deref, DerefMut};

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct PlayableAnimation(CircularAutoReversingAnimation);

impl PlayableAnimation {
    pub fn new(duration: usize) -> Self {
        Self(CircularAutoReversingAnimation::new(duration))
    }
    pub fn get_opacity(&self) -> f32 {
        // This only affects the last 15% of the opacity.
        self.progress(Easing::InOutCubic) * 0.15 + 0.85
    }
}
