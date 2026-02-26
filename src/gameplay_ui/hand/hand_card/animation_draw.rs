use crate::animation::{BasicAnimation, Easing};
use derive_more::{Deref, DerefMut};

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct DrawAnimation(BasicAnimation);

impl DrawAnimation {
    pub fn new(duration: usize) -> Self {
        Self(BasicAnimation::new(duration, false))
    }
    pub fn get_contraction(&self) -> f32 {
        self.progress(Easing::Linear)
    }
    pub fn get_scale(&self) -> f32 {
        // This animates only the last 50% of the scale.
        0.5 + self.progress(Easing::OutCubic) * 0.5
    }
}
