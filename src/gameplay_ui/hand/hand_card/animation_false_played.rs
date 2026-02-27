use crate::animation::{AutoReversingAnimation, Easing};
use derive_more::{Deref, DerefMut};

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct FalsePlayedAnimation(AutoReversingAnimation);

impl FalsePlayedAnimation {
    pub fn new(duration: usize) -> Self {
        Self(AutoReversingAnimation::new(duration))
    }
    pub fn get_opacity(&self) -> f32 {
        self.progress(Easing::InOutSine)
    }
}
