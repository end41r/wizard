use crate::{animation::{CircularAutoReversingAnimation, Easing, ReversableBasicAnimation}, api::{Card, get_glow_path}, client::{AppMessage, TaskBatcher}, gameplay_ui::{card_heigth_middle, card_width_middle}, ui_element_traits::{Animated, Resizable, Viewable}};
use derive_more::{Deref, DerefMut};
use iced::{Size, widget::{Container, image}};

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct RevealAnimation(ReversableBasicAnimation);

impl RevealAnimation {
    pub fn new(duration: usize) -> Self {
        Self(ReversableBasicAnimation::new(duration))
    }
    pub fn get_opacity(&self) -> f32 {
        self.progress(Easing::InSine)
    }
}

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct GlowAnimation(CircularAutoReversingAnimation);

impl GlowAnimation {
    pub fn new(duration: usize) -> Self {
        Self(CircularAutoReversingAnimation::new(duration))
    }
    pub fn get_opacity(&self) -> f32 {
        0.7 + 0.3 * self.progress(Easing::InSine)
    }
}

pub struct CardStackGlow {
    window_size: Size,
    img_path: String,
    pub reveal_animation: RevealAnimation,
    pub glow_animation: GlowAnimation,
}

impl CardStackGlow {
    pub fn new(window_size: Size) -> Self {
        let mut card_stack_glow = Self {
            window_size,
            img_path: "".to_string(),
            reveal_animation: RevealAnimation::new(250),
            glow_animation: GlowAnimation::new(60)
        };
        card_stack_glow.glow_animation.start();
        card_stack_glow
    }
    pub fn change_color(&mut self, card: Card) {
        self.img_path = get_glow_path(card);
    }
    pub fn reset_color(&mut self) {
        self.img_path = "".to_string()
    }
}

impl Animated for CardStackGlow {
    fn update_animations(&mut self) -> iced::Task<crate::client::AppMessage> {
        TaskBatcher::instant_batch([
            self.reveal_animation.next_frame(),
            self.glow_animation.next_frame()
        ])
    }
}

impl Resizable for CardStackGlow {
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

impl Viewable for CardStackGlow {
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let img = image(self.img_path.to_string())
            .width(self.width())
            .height(self.height())
            .opacity(self.glow_animation.get_opacity().min(self.reveal_animation.get_opacity()));
        Container::new(img)
    }
}