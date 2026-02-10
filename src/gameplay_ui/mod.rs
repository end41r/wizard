pub mod hand;
pub mod table;

use iced::Size;

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

// 1.5 to make a middle card appear a bit smaller than a hand card.
static CARD_MIDDLE_HAND_RELATION_MULT_WITH_WINDOW_WIDTH: f32 = 1.5;

// The factor 0.92 is chosen so the card img will not get clipped when rotated.
static CARD_IMG_BASE_SCALE: f32 = 0.92;

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
fn card_width_middle(window_size: Size) -> f32 {
    card_width_hand(window_size) * CARD_MIDDLE_HAND_RELATION_MULT_WITH_WINDOW_WIDTH
}
fn card_heigth_middle(window_size: Size) -> f32 {
    card_height_hand(window_size) * CARD_MIDDLE_HAND_RELATION_MULT_WITH_WINDOW_WIDTH
}
fn card_img_hand_base_scale() -> f32 {
    CARD_IMG_BASE_SCALE
}
fn card_img_table_base_scale() -> f32 {
    CARD_IMG_BASE_SCALE * (2.0 / 3.0)
}
