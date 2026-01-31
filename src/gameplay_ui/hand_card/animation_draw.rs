use crate::animation::animation::{BasicAnimation, new_basic};
use crate::animation::Easing;
use std::num::NonZero;
use derive_more::{Deref, DerefMut};

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct DrawAnimation(BasicAnimation);

new_basic!(DrawAnimation);

impl DrawAnimation {
    pub fn get_contraction(&self) -> f32 {
        self.progress(Easing::Linear)
    }
    pub fn get_scale(&self) -> f32 {
        // This animates only the last 50% of the scale.
        0.5 + self.progress(Easing::OutCubic) * 0.5
    }
}
