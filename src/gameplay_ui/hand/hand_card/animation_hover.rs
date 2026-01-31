use crate::animation::animation::ReversableBasicAnimation;
use crate::animation::Easing;
use crate::gameplay_ui::hand::hand_card::ViewableCard;
use crate::ui_element_traits::SizeFromOutside;
use derive_more::{Deref, DerefMut};
use iced::Size;
use std::num::NonZero;

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct HoverAnimation(ReversableBasicAnimation);

impl HoverAnimation {
    pub fn new(duration: NonZero<usize>) -> Self {
        Self(ReversableBasicAnimation::new(duration))
    }
    pub fn get_offset(&self, window_size: Size) -> f32 {
        ViewableCard::height_for(window_size) * 0.15 * self.progress(Easing::Linear)
    }
    // The factor 0.1 partially determines the hand width in hand::Hand::width_overflow_one_side.
    pub fn get_expansion(&self) -> f32 {
        1.0 + self.progress(Easing::Linear) * 0.1
    }
}
