use std::num::NonZero;
/// An AnimationStarter starts animations with a delay greater zero.
/// If you want to start multiple animations on the same time it is easier to do it manually.
/// Start it with the start function in an ui element impl Animated in update_with_msg
/// and start an animation with the check function in update animations.
/// Make sure that your AnimationStarters are at the beginning in update animations.
#[derive(Debug)]
pub struct AnimationStarter<C> {
    animation_delay: NonZero<usize>,
    tick: usize,
    times: usize,
    /// `Active` -> `true`
    ///
    /// `Inactive` -> `false`
    state: bool,
    content: Option<C>,
}

impl<C> AnimationStarter<C> {
    pub fn new(animation_delay: NonZero<usize>) -> Self {
        Self {
            animation_delay,
            tick: 0,
            times: 1, // This Will be set in the start function first before it matters.
            state: false,
            // You may need some additional information for the check function
            // you only know when you call the start function.
            content: None,
        }
    }
    // You may need some additional information for the check function
    // you only know when you call the start function.
    // Pass the information as content here to use it later on.
    // But make sure that the content and its context still represent what it should by then.
    pub fn start(&mut self, content: Option<C>, times: NonZero<usize>) {
        if !self.state {
            self.state = true;
        }
        // It will be easier to calculate later on starting with 0 and not 1.
        // Sow while times is non zero for the user it actually starts with 0 internally.
        self.times = times.get() - 1;
        self.content = content;
    }
    /// AI-Usage: Claude.ai for the implementation of passing a closure to a function
    ///           to execute it in the function.
    ///
    /// Use this every time update_animations from the Animated trait is called.
    ///
    /// This function executes the action used for starting an animation
    /// everytime the delay period has passed except for the first animation which is started
    /// immediately.
    ///
    /// This function returns true when the last tick is reached
    /// and all animations are started (NOT ended).
    /// You can use this property with an if-statement then to immediately execute an action.
    pub fn check<A>(&mut self, action: A) -> bool
    where
        A: FnOnce(&mut Self),
    {
        if self.state {
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
    #[allow(dead_code)]
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
    /// The number of the last started animation starting from 0.
    pub fn cycle(&self) -> usize {
        (self.tick - (self.tick % self.animation_delay)) / self.animation_delay
    }
    fn last_tick_reached(&self) -> bool {
        self.tick == self.times * self.animation_delay.get()
    }
}
