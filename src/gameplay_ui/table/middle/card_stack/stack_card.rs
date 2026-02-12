use crate::{
    animation::{BasicAnimation, Easing},
    api::{get_card_path, Card},
    client::AppMessage,
    gameplay_ui::{card_heigth_middle, card_img_table_base_scale, card_width_middle},
    ui_element_traits::*,
};

use derive_more::{Deref, DerefMut};
use iced::{
    widget::{image, Container},
    Size, Task,
};
use rand::Rng;

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct RevealAnimation(BasicAnimation);

impl RevealAnimation {
    pub fn new(duration: usize) -> Self {
        Self(BasicAnimation::new(duration))
    }
    pub fn get_rotation(&self) -> f32 {
        self.progress(Easing::InSine)
    }
    pub fn get_scale(&self) -> f32 {
        card_img_table_base_scale() * 1.5
            - (card_img_table_base_scale() / 2.0) * self.progress(Easing::OutElastic)
    }
}

pub struct ViewableStackCard {
    window_size: Size,
    card: Card,
    reveal_animation: RevealAnimation,
    rotation: f32,
}

impl ViewableStackCard {
    pub fn new(window_size: Size, card: Card) -> Self {
        let mut viewable_stack_card = Self {
            window_size,
            card,
            reveal_animation: RevealAnimation::new(50),
            rotation: rand::rng().random_range(-0.15..0.15),
        };
        viewable_stack_card.reveal_animation.start();
        viewable_stack_card
    }
}

impl Animated for ViewableStackCard {
    fn update_animations(&mut self) -> Task<AppMessage> {
        self.reveal_animation.next_frame()
    }
}

impl Resizable for ViewableStackCard {
    fn height(&self) -> f32 {
        card_heigth_middle(self.window_size)
    }
    fn width(&self) -> f32 {
        card_width_middle(self.window_size)
    }
    fn update_size(&mut self, window_size: Size) {
        self.window_size = window_size
    }
}

impl SizeFromOutside for ViewableStackCard {
    fn height_for(window_size: Size) -> f32 {
        card_heigth_middle(window_size)
    }
    fn width_for(window_size: Size) -> f32 {
        card_width_middle(window_size)
    }
}

impl Viewable for ViewableStackCard {
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let img = image(get_card_path(self.card))
            .width(self.width())
            .height(self.height())
            .scale(self.reveal_animation.get_scale())
            .rotation(self.rotation * self.reveal_animation.get_rotation());
        Container::new(img)
    }
}
