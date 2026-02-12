/// For animations you first need to create a struct (e.g. MyAnimation) with your fitting
/// Animation type (e.g. BasicAnimation, AutoReversingAnimation)
/// and derive Debug, Clone, Deref and DerefMut.
/// It should e.g. look like this:
///
/// use derive_more::{Deref, DerefMut};
/// #[derive(Debug, Clone, Deref, DerefMut)]
/// pub struct MyAnimation(BasicAnimation);
///
/// Now add the new function and functions that calculate what you want using the progress function
/// which gives you the relation between the current and the max frame number.
/// The progress function also takes an easing parameter as enum.
/// Use it to adjust the flow of your animation.
/// It should e.g. look like this:
///
/// impl MyAnimation {
///     pub fn new(duration: usize) -> Self {
///         Self(BasicAnimation::new(duration))
///     }
///     pub fn get_opacity(&self) -> f32 {
///         self.progress(Easing::EaseInCubic)
///     }
///
/// Now you can put the animation in your struct you want to animate by creating a new instance
/// via the new function.
/// Keep in mind that if you choose 0 for the animation duration the animation will always
/// count as finished (the progress function returns 1.0).
pub mod animation_starter;

use derive_more::{Deref, DerefMut};
use iced::Task;
use std::f32::consts::PI;

use crate::client::AppMessage;

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
    #[allow(dead_code)]
    OutSine,
    InOutSine,
    #[allow(dead_code)]
    OutElastic,
    #[allow(dead_code)]
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

// AI-Usage: Claude.ai for learning how to make a trait require another trait.
//           (Now this is not the case here anymore but it used to be).

#[derive(Clone, Debug)]
pub struct AnimationCore {
    max_frame_number: usize,
    current_frame_number: usize,
    animation_state: AnimationState,
    on_ping: Option<AppMessage>,
}

impl AnimationCore {
    fn new(duration: usize) -> Self {
        Self {
            max_frame_number: duration,
            current_frame_number: 0,
            animation_state: AnimationState::NotMoving,
            on_ping: None,
        }
    }
    pub fn start(&mut self) {
        if self.not_moving() || self.animation_state == AnimationState::Ended {
            self.animation_state = AnimationState::MovingForward;
        }
    }
    #[allow(dead_code)]
    pub fn interrupt(&mut self) {
        self.animation_state = AnimationState::NotMoving;
    }
    pub fn reset(&mut self) {
        self.current_frame_number = 0;
        self.animation_state = AnimationState::NotMoving;
    }
    /// This function represents the progress of the animation ranging from 0.0 to 1.0.
    /// Choose an easing type to manipulate the look of the animation to your liking.
    pub fn progress(&self, curve: Easing) -> f32 {
        let progress: f32 = if self.max_frame_number == 0 {
            1.0
        } else {
            self.current_frame_number as f32 / self.max_frame_number as f32
        };
        match curve {
            Easing::Linear => progress,
            Easing::InCubic => ease_in_cubic(progress),
            Easing::OutCubic => ease_out_cubic(progress),
            Easing::InOutCubic => ease_in_out_cubic(progress),
            Easing::InSine => ease_in_sine(progress),
            Easing::OutSine => ease_out_sine(progress),
            Easing::InOutSine => ease_in_out_sine(progress),
            Easing::OutElastic => ease_out_elastic(progress),
            Easing::OutBounce => ease_out_bounce(progress),
        }
    }
    #[allow(dead_code)]
    pub fn current_frame_number(&self) -> usize {
        self.current_frame_number
    }
    pub fn not_moving(&self) -> bool {
        self.animation_state == AnimationState::NotMoving
    }
    pub fn on_end(&mut self, msg: AppMessage) {
        self.on_ping = Some(msg)
    }
    /// Send a message when aa animation reaches a special point (e.g. end, new repetition)
    fn task_ping(&self) -> Task<AppMessage> {
        match &self.on_ping {
            Some(msg) => Task::done(msg.clone()),
            None => Task::none(),
        }
    }
}

// AI-Usage: Claude.ai to learn how to use a macro and partially generate the code regarding
//           the macro.
macro_rules! new_core {
    ($name:ident) => {
        impl $name {
            // This is marked as not used because CircularAnimation is as of now not used.
            #[allow(dead_code)]
            pub fn new(duration: usize) -> Self {
                Self(AnimationCore::new(duration))
            }
        }
    };
}

new_core!(BasicAnimation);
new_core!(CircularAnimation);
new_core!(ReversableBasicAnimation);
new_core!(AutoReversingAnimation);
new_core!(CircularAutoReversingAnimation);

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct BasicAnimation(AnimationCore);
impl BasicAnimation {
    pub fn next_frame(&mut self) -> Task<AppMessage> {
        if self.animation_state == AnimationState::MovingForward {
            if self.current_frame_number < self.max_frame_number {
                self.current_frame_number += 1;
            } else {
                self.animation_state = AnimationState::Ended;
                return self.task_ping();
            }
        }
        Task::none()
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deref, DerefMut)]
pub struct CircularAnimation(AnimationCore);
impl CircularAnimation {
    #[allow(dead_code)]
    pub fn next_frame(&mut self) -> Task<AppMessage> {
        if self.animation_state == AnimationState::MovingForward && self.max_frame_number != 0 {
            let frame_number_before = self.current_frame_number;
            self.current_frame_number = (self.current_frame_number + 1) % self.max_frame_number;
            let frame_number_after = self.current_frame_number;
            if !(frame_number_before < frame_number_after) {
                return self.task_ping();
            }
        }
        Task::none()
    }
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct ReversableBasicAnimation(AnimationCore);
impl ReversableBasicAnimation {
    pub fn reverse(&mut self) {
        self.animation_state = AnimationState::Reversing;
    }
    pub fn start_from_reverse(&mut self) {
        self.animation_state = AnimationState::Reversing;
        self.current_frame_number = self.max_frame_number
    }
    pub fn next_frame(&mut self) -> Task<AppMessage> {
        match self.animation_state {
            AnimationState::MovingForward => {
                if self.current_frame_number < self.max_frame_number {
                    self.current_frame_number += 1;
                } else {
                    self.animation_state = AnimationState::NotMoving;
                    return self.task_ping();
                }
            }
            AnimationState::Reversing => {
                if self.current_frame_number > 0 {
                    self.current_frame_number -= 1;
                } else {
                    self.animation_state = AnimationState::Ended;
                    return self.task_ping();
                }
            }
            _ => {}
        }
        Task::none()
    }
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct AutoReversingAnimation(AnimationCore);
impl AutoReversingAnimation {
    pub fn next_frame(&mut self) -> Task<AppMessage> {
        match self.animation_state {
            AnimationState::MovingForward => {
                if self.current_frame_number < self.max_frame_number {
                    self.current_frame_number += 1;
                } else {
                    self.animation_state = AnimationState::Reversing;
                    return self.task_ping();
                }
            }
            AnimationState::Reversing => {
                if self.current_frame_number > 0 {
                    self.current_frame_number -= 1;
                } else {
                    self.reset();
                    return self.task_ping();
                }
            }
            _ => {}
        }
        Task::none()
    }
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct CircularAutoReversingAnimation(AnimationCore);
impl CircularAutoReversingAnimation {
    pub fn next_frame(&mut self) -> Task<AppMessage> {
        match self.animation_state {
            AnimationState::MovingForward => {
                if self.current_frame_number < self.max_frame_number {
                    self.current_frame_number += 1;
                } else {
                    self.animation_state = AnimationState::Reversing;
                    return self.task_ping();
                }
            }
            AnimationState::Reversing => {
                if self.current_frame_number > 0 {
                    self.current_frame_number -= 1;
                } else {
                    self.animation_state = AnimationState::MovingForward;
                    return self.task_ping();
                }
            }
            _ => {}
        }
        Task::none()
    }
}
