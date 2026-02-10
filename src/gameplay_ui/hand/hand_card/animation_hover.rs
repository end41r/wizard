use crate::animation::{Easing, ReversableBasicAnimation};
use crate::gameplay_ui::hand::hand_card::ViewableHandCard;
use crate::ui_element_traits::SizeFromOutside;
use derive_more::{Deref, DerefMut};
use iced::Size;

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct HoverAnimation(ReversableBasicAnimation);

impl HoverAnimation {
    pub fn new(duration: usize) -> Self {
        Self(ReversableBasicAnimation::new(duration))
    }
    // The factor 0.15 changes the height of the hand in ViewableHand::height.
    pub fn get_offset(&self, window_size: Size) -> f32 {
        ViewableHandCard::height_for(window_size) * 0.15 * self.progress(Easing::Linear)
    }
    // The factor 0.1 partially determines the hand width in ViewableHand::width_overflow_one_side.
    pub fn get_expansion(&self) -> f32 {
        1.0 + self.progress(Easing::Linear) * 0.1
    }
}
