pub mod card_deck;
pub mod card_stack;

use crate::{
    client::AppMessage,
    gameplay_ui::table::{
        middle::{
            card_deck::{CardDeckMessage, ViewableCardDeck},
            card_stack::{CardStackMessage, ViewableCardStack},
        },
        TableMessage, ViewableTable,
    },
    ui_element_traits::*,
};
use iced::{
    widget::{row, Container},
    Size, Task,
};

#[derive(Debug, Clone)]
pub enum TableMiddleMessage {
    CardDeckMessage(CardDeckMessage),
    CardStackMessage(CardStackMessage),
}

pub struct ViewableTableMiddle {
    window_size: Size,
    card_deck: ViewableCardDeck,
    card_stack: ViewableCardStack,
}

impl ViewableTableMiddle {
    pub fn new(window_size: Size) -> Self {
        Self {
            window_size,
            card_deck: ViewableCardDeck::new(window_size),
            card_stack: ViewableCardStack::new(window_size),
        }
    }
}

impl Message for ViewableTableMiddle {
    type OwnMessage = TableMiddleMessage;
    fn convert_msg(msg: Self::OwnMessage) -> AppMessage {
        ViewableTable::convert_msg(TableMessage::TableMiddleMessage(msg))
    }
    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage> {
        let mut tasks: Vec<Task<AppMessage>> = vec![];
        match msg {
            TableMiddleMessage::CardDeckMessage(card_deck_msg) => {
                tasks.push(self.card_deck.update_with_msg(card_deck_msg))
            }
            TableMiddleMessage::CardStackMessage(card_stack_msg) => {
                tasks.push(self.card_stack.update_with_msg(card_stack_msg))
            }
        }
        Task::batch(tasks)
    }
}

impl Animated for ViewableTableMiddle {
    fn update_animations(&mut self) {
        self.card_deck.update_animations();
        self.card_stack.update_animations();
    }
}

impl Resizable for ViewableTableMiddle {
    fn height(&self) -> f32 {
        f32_max(vec![self.card_stack.height(), self.card_deck.height()]).unwrap()
    }
    fn width(&self) -> f32 {
        self.card_deck.width() + self.card_stack.width()
    }
    fn update_size(&mut self, window_size: Size) {
        self.window_size = window_size;
        self.card_deck.update_size(window_size);
        self.card_stack.update_size(window_size);
    }
}

impl Viewable for ViewableTableMiddle {
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        Container::new(row![self.card_stack.view(), self.card_deck.view()])
    }
}

fn f32_max(numbers: Vec<f32>) -> Option<f32> {
    let mut max: Option<f32> = None;
    for number in numbers.iter() {
        if max.is_none() {
            max = Some(*number)
        } else if max.unwrap() < *number {
            max = Some(*number)
        };
    }
    max
}
