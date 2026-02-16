#![allow(dead_code)]
pub mod hand;
pub mod scoreboard;
pub mod table;

use iced::{
    widget::{container, row, stack, Column, Container, Image},
    ContentFit, Point, Size, Task,
};

use crate::{
    client::{AppMessage, TaskBatcher},
    gameplay_ui::{
        hand::{HandMessage, ViewableHand},
        scoreboard::{ScoreBoard, ScoreBoardInfo, ScoreBoardMessage},
        table::{TableMessage, ViewableTable},
    },
    ui_element_traits::{Animated, Message, Notifiable, Resizable, Viewable},
};

// The hand size is depending on the window size with the factor 0.1.
static CARD_WIDTH_MULT_WITH_WINDOW_WIDTH: f32 = 0.1;
// 1.54 is around 1245 / 806 (height to width ratio of a card image).
pub static CARD_HEIGHT_MULT_WITH_WINDOW_WIDTH: f32 = CARD_WIDTH_MULT_WITH_WINDOW_WIDTH * 1.54;

// Adjust this arbitrary value to manipulate the width of the hand relative to its size,
// but be careful that the cards don't go out of screen.
// If you want to manipulate the total hand size
// change the value of hand_card::CARD_WIDTH_MULT_WITH_WINDOW_WIDTH.
static CARD_COLUMN_STEP_MULT_WITH_CARD_WIDTH: f32 = 1.0 / 3.0;
// Adjust this arbitrary value to manipulate the height of the hand,
static CARD_ROW_STEP_MULT_WITH_WINDOW_WIDTH: f32 = CARD_HEIGHT_MULT_WITH_WINDOW_WIDTH * 0.43;

// The factor is chosen so the card img will not get clipped when rotated.
static CARD_IMG_BASE_SCALE: f32 = 0.92;
static CARD_IMG_MIDDLE_BASE_SCALE: f32 = 0.8;

static CARD_AREA_MIDDLE_RELATION: f32 = 1.3;

fn card_width_hand(window_size: Size) -> f32 {
    CARD_WIDTH_MULT_WITH_WINDOW_WIDTH * window_size.width
}
fn card_height_hand(window_size: Size) -> f32 {
    CARD_HEIGHT_MULT_WITH_WINDOW_WIDTH * window_size.width
}
fn card_column_step_hand(card_width: f32) -> f32 {
    CARD_COLUMN_STEP_MULT_WITH_CARD_WIDTH * card_width
}
fn card_row_step_hand(window_size: Size) -> f32 {
    CARD_ROW_STEP_MULT_WITH_WINDOW_WIDTH * window_size.width
}
fn card_img_base_scale() -> f32 {
    CARD_IMG_BASE_SCALE
}
fn card_img_middle_base_scale() -> f32 {
    CARD_IMG_MIDDLE_BASE_SCALE
}
fn card_width_middle(window_size: Size) -> f32 {
    card_width_hand(window_size) * (card_img_base_scale() / card_img_middle_base_scale())
}
fn card_height_middle(window_size: Size) -> f32 {
    card_height_hand(window_size) * (card_img_base_scale() / card_img_middle_base_scale())
}
fn card_area_middle_space_width(window_size: Size) -> f32 {
    card_width_middle(window_size) * CARD_AREA_MIDDLE_RELATION
}
fn card_area_middle_space_height(window_size: Size) -> f32 {
    card_height_middle(window_size) * CARD_AREA_MIDDLE_RELATION
}
fn card_area_middle_spawn_point(width: f32, height: f32, window_size: Size) -> Point {
    Point::new(
        (card_area_middle_space_width(window_size) - width) / 2.0,
        (card_area_middle_space_height(window_size) - height) / 2.0,
    )
}

#[derive(Clone, Debug)]
pub enum GameViewMessage {
    HandMessage(HandMessage),
    TableMessage(TableMessage),
    ScoreBoardMessage(ScoreBoardMessage),
}

impl Message for GameViewMessage {
    fn convert_msg_from(msg: Self) -> crate::client::AppMessage {
        AppMessage::GameViewMessage(msg)
    }
}

pub struct GameView {
    window_size: Size,
    viewable_hand: ViewableHand,
    viewable_table: ViewableTable,
    scoreboard: ScoreBoard,
}

impl GameView {
    pub fn new(window_size: Size) -> Self {
        Self {
            window_size,
            viewable_hand: ViewableHand::new(window_size),
            viewable_table: ViewableTable::new(window_size),
            scoreboard: ScoreBoard::new(window_size, ScoreBoardInfo::default()),
        }
    }
}

impl Notifiable for GameView {
    type OwnMessage = GameViewMessage;
    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage> {
        match msg {
            GameViewMessage::HandMessage(hand_msg) => {
                return self.viewable_hand.update_with_msg(hand_msg)
            }
            GameViewMessage::TableMessage(table_msg) => {
                return self.viewable_table.update_with_msg(table_msg)
            }
            GameViewMessage::ScoreBoardMessage(sb_msg) => {
                return self.scoreboard.update_with_msg(sb_msg)
            }
        }
    }
}

impl Animated for GameView {
    fn update_animations(&mut self) -> Task<AppMessage> {
        TaskBatcher::instant_batch([
            self.viewable_hand.update_animations(),
            self.viewable_table.update_animations(),
        ])
    }
}

impl Resizable for GameView {
    fn height(&self) -> f32 {
        self.window_size.height
    }
    fn width(&self) -> f32 {
        self.window_size.width
    }
    fn update_size(&mut self, window_size: Size) {
        self.window_size = window_size;
        self.viewable_hand.update_size(window_size);
        self.viewable_table.update_size(window_size);
    }
}

impl Viewable for GameView {
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let scoreboard = self.scoreboard.view();

        let main_content = Column::new()
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);

        let content = row![main_content, scoreboard,].height(iced::Length::Fill);

        Container::new(stack![
            Image::new("assets/ingame_background.png")
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .content_fit(ContentFit::Cover),
            container(content)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill),
        ])
    }
}
