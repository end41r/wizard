pub mod hand_card;

pub mod animation_draw;
pub mod animation_hover_focus;
pub mod animation_hover;
pub mod animation_play;
pub mod animtion_hide;

pub fn f32_min_2(v1: f32, v2: f32) -> f32 {
    if v1 < v2 {
        return v1
    }
    v2
}

pub fn f32_min_3(v1: f32, v2: f32, v3: f32) -> f32 {
    f32_min_2(f32_min_2(v1, v2), v3)
}
