use crate::{
    api::get_card_path,
    client::AppMessage,
    gameplay_ui::{
        card_heigth_middle, card_img_table_base_scale, card_width_middle,
        table::middle::card_deck::Card,
    },
    ui_element_traits::*,
};
use iced::{
    widget::{image, Container},
    Size, Task,
};

pub struct ViewableTrumpCard {
    window_size: Size,
    trump_card: Card,
}

impl ViewableTrumpCard {
    pub fn new(window_size: Size, trump_card: Card) -> Self {
        Self {
            window_size,
            trump_card,
        }
    }
}

impl Animated for ViewableTrumpCard {
    fn update_animations(&mut self) -> Task<AppMessage> {
        Task::none()
    }
}

impl Resizable for ViewableTrumpCard {
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

impl Viewable for ViewableTrumpCard {
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let img = image(get_card_path(self.trump_card))
            .width(self.width() * card_img_table_base_scale())
            .height(self.height() * card_img_table_base_scale());
        Container::new(img)
    }
}
