use crate::animation::{Easing, ReversableBasicAnimation};
use derive_more::{Deref, DerefMut};

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct HideAnimation(ReversableBasicAnimation);

impl HideAnimation {
    pub fn new(duration: usize) -> Self {
        Self(ReversableBasicAnimation::new(duration))
    }
    pub fn get_opacity(&self) -> f32 {
        1.0 - self.progress(Easing::Linear)
    }
    pub fn get_contraction(&self) -> f32 {
        1.0 - self.progress(Easing::Linear)
    }
    pub fn get_scale(&self) -> f32 {
        1.0 - self.progress(Easing::Linear)
    }
}
