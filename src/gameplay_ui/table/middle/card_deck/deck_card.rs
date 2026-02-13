use crate::{
    animation::{Easing, ReversableBasicAnimation},
    api::CARD_BACK_PATH,
    client::AppMessage,
    gameplay_ui::{
        card_heigth_hand, card_heigth_middle, card_img_middle_base_scale, card_width_hand,
        card_width_middle,
    },
    ui_element_traits::*,
};
use derive_more::{Deref, DerefMut};
use iced::{
    widget::{image, Container},
    Point, Size, Task,
};

#[derive(Debug, Clone)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct DealAnimation(ReversableBasicAnimation);

impl DealAnimation {
    fn new(duration: usize) -> Self {
        Self(ReversableBasicAnimation::new(duration))
    }
    fn get_offset(&self) -> f32 {
        self.progress(Easing::InSine)
    }
    fn get_opacity(&self) -> f32 {
        1.0 - self.progress(Easing::InSine)
    }
}

#[derive(Debug, Clone)]
pub struct ViewableDeckCard {
    window_size: Size,
    add: bool,
    direction: Direction,
    deal_animation: DealAnimation,
}

impl ViewableDeckCard {
    pub fn new(window_size: Size, cycle: usize, add: bool) -> Self {
        let mut viewable_deck_card = Self {
            window_size,
            add,
            direction: Self::choose_direction(cycle),
            deal_animation: DealAnimation::new(10),
        };
        viewable_deck_card.deal_animation.start();
        viewable_deck_card
    }
    pub fn offset(&self) -> Point {
        let mut linear_progress: f32 = self.deal_animation.get_offset();
        if self.add {
            linear_progress = 1.0 - linear_progress;
        }
        let horizontal_offset: f32 = linear_progress * card_width_hand(self.window_size) / 6.0;
        let vertical_offset: f32 = linear_progress * card_heigth_hand(self.window_size) / 6.0;
        match self.direction {
            Direction::Down => Point::new(0.0, linear_progress * horizontal_offset),
            Direction::Left => Point::new(-linear_progress * vertical_offset, 0.0),
            Direction::Right => Point::new(linear_progress * vertical_offset, 0.0),
            Direction::Up => Point::new(0.0, -linear_progress * horizontal_offset),
        }
    }
    fn choose_direction(cycle: usize) -> Direction {
        let direction_number: usize = cycle % 4;
        match direction_number {
            0 => Direction::Left,
            1 => Direction::Up,
            2 => Direction::Right,
            _ => Direction::Down,
        }
    }
}

impl Animated for ViewableDeckCard {
    fn update_animations(&mut self) -> Task<AppMessage> {
        self.deal_animation.next_frame()
    }
}

impl Resizable for ViewableDeckCard {
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

impl SizeFromOutside for ViewableDeckCard {
    fn height_for(window_size: Size) -> f32 {
        card_heigth_middle(window_size)
    }
    fn width_for(window_size: Size) -> f32 {
        card_width_middle(window_size)
    }
}

impl Viewable for ViewableDeckCard {
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let opacity = if self.add {1.0 - self.deal_animation.get_opacity()} else {self.deal_animation.get_opacity()};
        let img = image(CARD_BACK_PATH.to_string())
            .width(self.width())
            .height(self.height())
            .scale(card_img_middle_base_scale())
            .opacity(opacity);
        Container::new(img)
    }
}
