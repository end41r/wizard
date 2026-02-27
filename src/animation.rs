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
///
/// You can also set a Message to be sent when the animation reaches the highest frame number
/// via on_end or returns(!) to the first frame number 0 via on start.
/// Check the source of your animation type for the specific implementation of this.
///
/// If you want to start multiple animations with a set delay you need to use AnimationStarter.
/// You give it a message with an usize 0 at its end that will start an animation.
/// The message needs to implement ReplaceUsize + Message for this.
/// You can also give it a message to send when all animations have ended via on_all_ended.
/// Now to use it you call start and update it every animation tick via next_frame
use derive_more::{Deref, DerefMut};
use iced::Task;
use std::f32::consts::PI;

use crate::{
    client::AppMessage,
    ui_element_traits::{Message, ReplaceUsize},
};

#[derive(Debug, PartialEq, Clone, Copy)]
enum AnimationState {
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
    after_max_frame_number: usize,
    current_frame_number: usize,
    animation_state: AnimationState,
    on_start: Option<AppMessage>,
    on_end: Option<AppMessage>,
}

impl AnimationCore {
    fn new(duration: usize) -> Self {
        Self {
            after_max_frame_number: duration,
            current_frame_number: 0,
            animation_state: AnimationState::NotMoving,
            on_start: None,
            on_end: None,
        }
    }
    pub fn start(&mut self) {
        self.animation_state = AnimationState::MovingForward;
    }
    pub fn start_force(&mut self) {
        self.current_frame_number = 0;
        self.animation_state = AnimationState::MovingForward;
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
        let progress: f32 = if self.after_max_frame_number == 0 {
            1.0
        } else {
            self.current_frame_number as f32 / self.after_max_frame_number as f32
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
    pub fn current_frame_number(&self) -> usize {
        self.current_frame_number
    }
    pub fn max_frame_number(&self) -> usize {
        if self.after_max_frame_number == 0 {
            0
        } else {
            self.after_max_frame_number - 1
        }
    }
    /// Message sent when an animation reaches an end point
    pub fn on_end_reached(&mut self, msg: AppMessage) {
        self.on_end = Some(msg)
    }
    pub fn on_start_reached(&mut self, msg: AppMessage) {
        self.on_start = Some(msg)
    }
    /// Send a message when an animation reaches a reverse point or end if not reversing.
    fn end_task(&self) -> Task<AppMessage> {
        match &self.on_end {
            Some(msg) => Task::done(msg.clone()),
            None => Task::none(),
        }
    }
    /// Send a message everytime when an animation returns to start.
    /// This won't trigger when the animation is started.
    fn start_reached_task(&self) -> Task<AppMessage> {
        match &self.on_start {
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
            if self.current_frame_number < self.after_max_frame_number {
                self.current_frame_number += 1;
            } else {
                self.animation_state = AnimationState::Ended;
                return self.end_task();
            }
        }
        Task::none()
    }
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct CircularAnimation(AnimationCore);
impl CircularAnimation {
    pub fn start_infinite(&mut self) {
        self.animation_state = AnimationState::MovingForward;
    }
    pub fn next_frame(&mut self) -> Task<AppMessage> {
        if self.animation_state == AnimationState::MovingForward && self.after_max_frame_number != 0
        {
            let frame_number_before = self.current_frame_number;
            self.current_frame_number =
                (self.current_frame_number + 1) % self.after_max_frame_number;
            let frame_number_after = self.current_frame_number;
            if frame_number_before >= frame_number_after {
                return self.end_task();
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
    #[allow(dead_code)]
    pub fn reverse_force(&mut self) {
        self.current_frame_number = self.after_max_frame_number;
        self.animation_state = AnimationState::Reversing;
    }
    pub fn next_frame(&mut self) -> Task<AppMessage> {
        match self.animation_state {
            AnimationState::MovingForward => {
                if self.current_frame_number < self.after_max_frame_number {
                    self.current_frame_number += 1;
                } else {
                    self.animation_state = AnimationState::NotMoving;
                    return self.end_task();
                }
            }
            AnimationState::Reversing => {
                if self.current_frame_number > 0 {
                    self.current_frame_number -= 1;
                } else {
                    self.animation_state = AnimationState::Ended;
                    return self.start_reached_task();
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
                if self.current_frame_number < self.after_max_frame_number {
                    self.current_frame_number += 1;
                } else {
                    self.animation_state = AnimationState::Reversing;
                    return self.end_task();
                }
            }
            AnimationState::Reversing => {
                if self.current_frame_number > 0 {
                    self.current_frame_number -= 1;
                } else {
                    self.current_frame_number = 0;
                    self.animation_state = AnimationState::NotMoving;
                    return self.start_reached_task();
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
    pub fn start_infinite(&mut self) {
        self.animation_state = AnimationState::MovingForward;
    }
    pub fn next_frame(&mut self) -> Task<AppMessage> {
        match self.animation_state {
            AnimationState::MovingForward => {
                if self.current_frame_number < self.after_max_frame_number {
                    self.current_frame_number += 1;
                } else {
                    self.animation_state = AnimationState::Reversing;
                    return self.end_task();
                }
            }
            AnimationState::Reversing => {
                if self.current_frame_number > 0 {
                    self.current_frame_number -= 1;
                } else {
                    self.animation_state = AnimationState::MovingForward;
                    return self.start_reached_task();
                }
            }
            _ => {}
        }
        Task::none()
    }
}

#[derive(Clone, Debug)]
pub struct AnimationStarter<MStart: Message + ReplaceUsize, MEnd: Message> {
    state: AnimationState,
    times: usize,
    animation_delay: usize,
    animation_length: usize,
    tick: usize,
    started: usize,
    on_start_single: MStart,
    on_all_ended: Option<MEnd>,
}

impl<MStart: Message + ReplaceUsize, MEnd: Message> AnimationStarter<MStart, MEnd> {
    /// Check AppMessage::replace_usize
    pub fn new(animation_delay: usize, animation_duration: usize, first_start_msg: MStart) -> Self {
        Self {
            times: 0,
            state: AnimationState::NotMoving,
            animation_delay,
            animation_length: animation_duration,
            tick: 0,
            started: 0,
            on_start_single: first_start_msg,
            on_all_ended: None,
        }
    }
    pub fn start(&mut self, times: usize) -> Task<AppMessage> {
        self.times = times;
        self.state = AnimationState::MovingForward;
        Task::none()
    }
    pub fn next_frame(&mut self) -> Task<AppMessage> {
        if self.state == AnimationState::MovingForward {
            self.tick += 1;
            if self.check_all_ended() {
                return self.all_ended();
            } else if self.check_tick() {
                return self.start_single();
            }
        }
        Task::none()
    }
    pub fn on_all_ended(&mut self, msg: MEnd) {
        self.on_all_ended = Some(msg)
    }
    pub fn times(&self) -> usize {
        self.times
    }
    fn check_tick(&self) -> bool {
        self.tick.is_multiple_of(self.animation_delay)
            && self.tick <= self.animation_delay * self.times
    }
    fn check_all_ended(&self) -> bool {
        (self.tick == self.animation_delay * self.times + self.animation_length)
            || (self.times == 0)
    }
    fn all_ended(&mut self) -> Task<AppMessage> {
        let end_task: Task<AppMessage> = match &self.on_all_ended {
            Some(msg) => Task::done(msg.convert_msg()),
            None => Task::none(),
        };
        self.state = AnimationState::NotMoving;
        self.tick = 0;
        self.started = 0;
        end_task
    }
    fn start_single(&mut self) -> Task<AppMessage> {
        self.started += 1;
        self.on_start_single
            .replace_usize(self.started - 1)
            .convert_msg_to_task()
    }
}

#[derive(Clone, Debug)]
pub struct AnimationChainGuardian {
    tick: usize,
    max_tick: usize,
    is_moving: bool,
}

impl AnimationChainGuardian {
    pub fn new() -> Self {
        Self {
            tick: 0,
            max_tick: 0,
            is_moving: false,
        }
    }
    pub fn start(&mut self, chain_duration: usize) -> Task<AppMessage> {
        self.is_moving = true;
        self.tick = 0;
        self.max_tick = chain_duration;
        AppMessage::IncrementACDL.convert_msg_to_task()
    }
    pub fn next_frame(&mut self) -> Task<AppMessage> {
        if self.is_moving {
            if self.tick < self.max_tick {
                self.tick += 1;
            } else {
                self.is_moving = false;
                self.tick = 0;
                self.max_tick = 0;
                return AppMessage::DecrementACDL.convert_msg_to_task();
            }
        }
        Task::none()
    }
}
