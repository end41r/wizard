pub mod hand_card;

pub mod animation_draw;
pub mod animation_hover_focus;
pub mod animation_hover;
pub mod animation_play;
pub mod animation_playable;
pub mod animation_false_played;
pub mod animation_hide;

pub fn f32_min_2(v1: f32, v2: f32) -> f32 {
    if v1 < v2 {
        return v1
    }
    v2
}