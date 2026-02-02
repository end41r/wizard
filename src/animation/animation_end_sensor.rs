/// An AnimationEndSensor executes a code block when an animation ends.
/// Start the sensor with the start function in an ui element impl Animated in update_with_msg
/// and execute the code with the check function in update animations.
/// Make sure that your AnimationEndSensors are at the end in update animations.
#[derive(Debug)]
pub struct AnimationEndSensor<C> {
    animation_duration: usize,
    tick: usize,
    /// `Active` -> `true`
    /// 
    /// `Inactive` -> `false`
    state: bool,
    content: Option<C>,
}

impl<C> AnimationEndSensor<C> {
    pub fn new(animation_duration: usize) -> Self {
        Self {
            animation_duration,
            tick: 0,
            state: false,
            content: None,
        }
    }
    // You may need some additional information for the check function
    // you only know when you call the start function.
    // Pass the information as content here to use it later on.
    // But make sure that the content and its context still represent what it should by then.
    pub fn start(&mut self, content: Option<C>) {
        if !self.state {
            self.state = true;
        }
        self.content = content;
    }
    /// AI-Usage: Claude.ai for the implementation of passing a closure to a function
    ///           to execute it in the function.
    ///
    /// Use this every time update_animations from the Animated trait is called.
    ///
    /// This function executes the action when the last tick is reached.
    ///
    /// This function returns true when the last tick is reached.
    /// This property is useful when facing borrowing issues within the action.
    pub fn check<A>(&mut self, action: A) -> bool
    where
        A: FnOnce(&mut Self),
    {
        if self.state {
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
    pub fn content(&self) -> Option<&C> {
        self.content.as_ref()
    }
    pub fn active(&self) -> bool {
        self.state
    }
    pub fn reset(&mut self) {
        self.state = false;
        self.tick = 0;
    }
    fn last_tick_reached(&self) -> bool {
        self.tick == self.animation_duration
    }
}
