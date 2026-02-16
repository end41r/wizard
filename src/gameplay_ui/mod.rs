#![allow(dead_code)]
pub mod hand;
pub mod scoreboard;
pub mod table;

use iced::{Point, Size};

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
