use std::num::NonZero;

#[derive(PartialEq, Debug)]
enum AnimationStarterState {
    Active,
    Inactive,
}

/// An AnimationStarter is used for starting multiple animations with a delay.
/// It allows you to execute a code block when the last animation has started (NOT ended).
#[derive(Debug)]
pub struct AnimationStarter<C> {
    animation_delay: NonZero<usize>,
    tick: usize,
    times: usize,
    state: AnimationStarterState,
    content: Option<C>
}

impl<C> AnimationStarter<C> {
    pub fn new(delay: NonZero<usize>) -> Self {
        Self {
            animation_delay: delay,
            tick: 0,
            times: 0,  // Will be set in fn start() where it first matters
            state: AnimationStarterState::Inactive,
            content: None
        }
    }
    pub fn content(&self) -> Option<&C> {
        self.content.as_ref()
    }
    pub fn active(&self) -> bool {
        match self.state {
            AnimationStarterState::Active => true,
            AnimationStarterState::Inactive => false
        }
    }
    pub fn last_tick_reached(& self) -> bool {
        self.tick == self.times * self.animation_delay.get()
    }
    pub fn start(&mut self, content: Option<C>, times: usize) {
        if self.state == AnimationStarterState::Inactive {
            self.state = AnimationStarterState::Active
        }
        self.times = if times == 0 {times} else {times - 1};
        self.content = content;
    }
    pub fn reset(&mut self) {
        self.state = AnimationStarterState::Inactive;
        self.tick = 0;
    }
    /// Use this every time update_animations from the GameElement trait is called.
    /// 
    /// This function executes the action used for starting an animation
    /// everytime the delay period has passed.
    /// 
    /// This function returns true when the last tick is reached
    /// and all animations are started (NOT ended).
    /// You can use this property with an if-statement then to immediately execute an action.
    pub fn check<A>(&mut self, action: A) -> bool where A: FnOnce(&mut Self) {
        if self.state == AnimationStarterState::Active {
            if self.tick % self.animation_delay == 0 {
                action(self);
            }
            if self.last_tick_reached() {
                self.reset();
                return true;
            } else {
                self.tick += 1;
            }
        };
        false
    }
    pub fn cycle(&self) -> usize {
        (self.tick - (self.tick % self.animation_delay)) / self.animation_delay
    }
}