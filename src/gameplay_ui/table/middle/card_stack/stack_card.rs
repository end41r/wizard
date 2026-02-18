use crate::{
    animation::{BasicAnimation, Easing},
    api::Card,
    client::{AppMessage, TaskBatcher},
    gameplay_ui::{
        card_height_middle, card_img_middle_base_scale, card_width_middle,
        CARD_AREA_MIDDLE_RELATION,
    },
    ui_element_traits::*,
};

use derive_more::{Deref, DerefMut};
use iced::{
    widget::{image, Container},
    ContentFit::Fill,
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
        CARD_AREA_MIDDLE_RELATION
            - (CARD_AREA_MIDDLE_RELATION - 1.0) * self.progress(Easing::OutElastic)
    }
}

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct RemoveAnimation(BasicAnimation);

impl RemoveAnimation {
    pub fn new(duration: usize) -> Self {
        Self(BasicAnimation::new(duration))
    }
    pub fn get_opacity(&self) -> f32 {
        1.0 - self.progress(Easing::OutCubic)
    }
    pub fn get_contraction(&self) -> f32 {
        1.0 - self.progress(Easing::OutCubic)
    }
}

pub struct ViewableStackCard {
    window_size: Size,
    card: Card,
    reveal_animation: RevealAnimation,
    pub remove_animation: RemoveAnimation,
    rotation: f32,
}

impl ViewableStackCard {
    pub fn new(window_size: Size, card: Card) -> Self {
        let mut viewable_stack_card = Self {
            window_size,
            card,
            reveal_animation: RevealAnimation::new(50),
            remove_animation: RemoveAnimation::new(10),
            rotation: rand::rng().random_range(-0.15..0.15),
        };
        viewable_stack_card.reveal_animation.start();
        viewable_stack_card
    }
}

impl Animated for ViewableStackCard {
    fn update_animations(&mut self) -> Task<AppMessage> {
        TaskBatcher::instant_batch([
            self.reveal_animation.next_frame(),
            self.remove_animation.next_frame(),
        ])
    }
}

impl Resizable for ViewableStackCard {
    fn height(&self) -> f32 {
        card_height_middle(self.window_size) * self.reveal_animation.get_scale()
    }
    fn width(&self) -> f32 {
        card_width_middle(self.window_size)
            * self.reveal_animation.get_scale()
            * self.remove_animation.get_contraction()
    }
    fn update_size(&mut self, window_size: Size) {
        self.window_size = window_size
    }
}

impl SizeFromOutside for ViewableStackCard {
    fn height_for(window_size: Size) -> f32 {
        card_height_middle(window_size)
    }
    fn width_for(window_size: Size) -> f32 {
        card_width_middle(window_size)
    }
}

impl Viewable for ViewableStackCard {
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let img = image(self.card.img_path())
            .width(self.width())
            .height(self.height())
            .scale(card_img_middle_base_scale())
            .opacity(self.remove_animation.get_opacity())
            .content_fit(Fill)
            .rotation(self.rotation * self.reveal_animation.get_rotation());
        Container::new(img)
    }
}
