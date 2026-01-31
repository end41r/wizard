use std::f32::consts::PI;

pub mod animation;
pub mod animation_end_sensor;
pub mod animation_starter;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AnimationState {
    NotMoving,
    MovingForward,
    Reversing,
    Ended,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Easing {
    Linear,
    InCubic,
    OutCubic,
    InOutCubic,
    InSine,
    OutSine,
    InOutSine,
    OutElastic,
    OutBounce,
}

// AI-Usage: Claude.ai for generating the logic of this function.
pub fn ease_in_cubic(v: f32) -> f32 {
    v * v * v
}

// AI-Usage: Claude.ai for generating the logic of this function.
pub fn ease_out_cubic(v: f32) -> f32 {
    1.0 - (1.0 - v) * (1.0 - v) * (1.0 - v)
}

// AI-Usage: Claude.ai for generating the logic of this function.
pub fn ease_in_out_cubic(v: f32) -> f32 {
    if v < 0.5 {
        4.0 * v * v * v
    } else {
        1.0 - ((-2.0 * v + 2.0) * (-2.0 * v + 2.0) * (-2.0 * v + 2.0)) / 2.0
    }
}

// AI-Usage: Claude.ai for generating the logic of this function.
pub fn ease_in_sine(v: f32) -> f32 {
    1.0 - ((v * PI) / 2.0).cos()
}

// AI-Usage: Claude.ai for generating the logic of this function.
pub fn ease_out_sine(v: f32) -> f32 {
    ((v * PI) / 2.0).sin()
}

// AI-Usage: Claude.ai for generating the logic of this function.
pub fn ease_in_out_sine(v: f32) -> f32 {
    -((v * PI).cos() - 1.0) / 2.0
}

// AI-Usage: Claude.ai for generating the code of this function.
pub fn ease_out_elastic(v: f32) -> f32 {
    const C: f32 = (2.0 * PI) / 3.0;
    if v == 0.0 {
        0.0
    } else if v == 1.0 {
        1.0
    } else {
        f32::powf(2.0, -10.0 * v) * ((v * 10.0 - 0.75) * C).sin() + 1.0
    }
}

// AI-Usage: Claude.ai for generating the code of this function.
pub fn ease_out_bounce(v: f32) -> f32 {
    const N: f32 = 7.5625;
    const D: f32 = 2.75;

    if v < 1.0 / D {
        N * v * v
    } else if v < 2.0 / D {
        let v: f32 = v - 1.5 / D;
        N * v * v + 0.75
    } else if v < 2.5 / D {
        let v: f32 = v - 2.25 / D;
        N * v * v + 0.9375
    } else {
        let v: f32 = v - 2.625 / D;
        N * v * v + 0.984375
    }
}
