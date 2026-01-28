use std::num::NonZero;

#[derive(PartialEq, Debug)]
enum AnimationEndSensorState {
    Active,
    Inactive,
}

/// An AnimationEndSensor executes a caode block when an animation ends.
/// Start the sensor with fn start and execute the code with fn check.
#[derive(Debug)]
pub struct AnimationEndSensor<C> {
    animation_length: NonZero<usize>,
    tick: usize,
    state: AnimationEndSensorState,
    content: Option<C>
}

impl<C> AnimationEndSensor<C> {
    pub fn new(duration: NonZero<usize>) -> Self {
        Self {
            animation_length: duration,
            tick: 0,
            state: AnimationEndSensorState::Inactive,
            content: None
        }
    }
    pub fn content(&self) -> Option<&C> {
        self.content.as_ref()
    }
    pub fn active(&self) -> bool {
        match self.state {
            AnimationEndSensorState::Active => true,
            AnimationEndSensorState::Inactive => false
        }
    }
    pub fn last_tick_reached(&self) -> bool {
        self.tick == self.animation_length.get()
    }
    pub fn start(&mut self, content: Option<C>) {
        if self.state == AnimationEndSensorState::Inactive {
            self.state = AnimationEndSensorState::Active
        }
        self.content = content;
    }
    pub fn reset(&mut self) {
        self.state = AnimationEndSensorState::Inactive;
        self.tick = 0;
    }
    /// Use this every time update_animations from the GameElement trait is called.
    /// 
    /// This function executes the action when the last tick is reached.
    /// 
    /// This function returns true when the last tick is reached.
    /// This property is useful when facing borrowing issues within the action.
    pub fn check<A>(&mut self, action: A) -> bool where A: FnOnce(&mut Self) {
        if self.state == AnimationEndSensorState::Active {
            if self.last_tick_reached() {
                action(self);
                self.reset();
                return true;
            } else {
                self.tick += 1;
            };
        }
        false
    }
}