use crate::{
    animation::{Easing, ReversableBasicAnimation},
    api::CARD_BACK_PATH,
    client::{AppMessage, TaskBatcher},
    gameplay_ui::{
        card_height_middle, card_img_middle_base_scale, card_width_middle,
        table::middle::card_deck::{Card, CardDeckMessage},
    },
    ui_element_traits::*,
};
use derive_more::{Deref, DerefMut};
use iced::{
    widget::{image, Container},
    Size, Task,
};

#[derive(Debug, Clone)]
pub enum TrumpCardMessage {
    TurnPart1,
    TurnPart2,
    RemovePart1,
    RemovePart2,
}

impl Message for TrumpCardMessage {
    fn convert_msg_from(msg: Self) -> AppMessage {
        CardDeckMessage::convert_msg_from(CardDeckMessage::TrumpCardMessage(msg))
    }
}

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct TurnAnimation(ReversableBasicAnimation);

impl TurnAnimation {
    fn new(duration: usize) -> Self {
        Self(ReversableBasicAnimation::new(duration))
    }
    fn get_contraction(&self) -> f32 {
        1.0 - self.progress(Easing::InSine)
    }
}

#[derive(Debug, Clone)]
pub struct ViewableTrumpCard {
    window_size: Size,
    trump_card: Card,
    show_back: bool,
    reveal_animation: TurnAnimation,
    remove_animation: TurnAnimation,
}

impl ViewableTrumpCard {
    pub fn new(window_size: Size, trump_card: Card) -> Self {
        let mut viewable_trump_card = Self {
            window_size,
            trump_card,
            show_back: true,
            reveal_animation: TurnAnimation::new(10),
            remove_animation: TurnAnimation::new(10),
        };
        viewable_trump_card
            .reveal_animation
            .on_end_reached(TrumpCardMessage::TurnPart2.convert_msg());
        viewable_trump_card
            .remove_animation
            .on_end_reached(TrumpCardMessage::RemovePart2.convert_msg());
        viewable_trump_card
            .remove_animation
            .on_start_reached(CardDeckMessage::ClearTrumpCard.convert_msg());
        viewable_trump_card
    }
}

impl Notifiable for ViewableTrumpCard {
    type OwnMessage = TrumpCardMessage;
    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage> {
        match msg {
            TrumpCardMessage::TurnPart1 => self.reveal_animation.start(),
            TrumpCardMessage::TurnPart2 => {
                self.show_back = false;
                self.reveal_animation.reverse();
            }
            TrumpCardMessage::RemovePart1 => self.remove_animation.start(),
            TrumpCardMessage::RemovePart2 => {
                self.show_back = true;
                self.remove_animation.reverse();
            }
        }
        Task::none()
    }
}

impl Animated for ViewableTrumpCard {
    fn update_animations(&mut self) -> Task<AppMessage> {
        TaskBatcher::instant_batch([
            self.reveal_animation.next_frame(),
            self.remove_animation.next_frame(),
        ])
    }
}

impl Resizable for ViewableTrumpCard {
    fn height(&self) -> f32 {
        card_height_middle(self.window_size)
    }
    fn width(&self) -> f32 {
        card_width_middle(self.window_size)
            * self.reveal_animation.get_contraction()
            * self.remove_animation.get_contraction()
    }
    fn update_size(&mut self, window_size: Size) {
        self.window_size = window_size
    }
}

impl Viewable for ViewableTrumpCard {
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let img_path = if !self.show_back {
            self.trump_card.img_path()
        } else {
            CARD_BACK_PATH.to_string()
        };
        let img = image(img_path)
            .scale(card_img_middle_base_scale())
            .content_fit(iced::ContentFit::Fill);
        Container::new(img)
            .width(self.width())
            .height(self.height())
    }
}
