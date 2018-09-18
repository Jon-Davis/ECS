use dispatcher::Dispatcher;
use systems::System;
use resources::Resources;

/// Represents a Transtion allowing one state
/// to transition into another state
pub enum Trans {
    None,
    Pop,
    Push(State),
    Swap(State),
}

/// A state has a dispatcher with a series of 
pub struct State {
    dispatcher: Dispatcher,
}

impl State {
    /// Creates a new state
    pub fn new() -> State {
        State {
            dispatcher: Dispatcher::new(),
        }
    }

    /// Adds a new system to the states dispatcher
    pub fn with(mut self, system : System) -> State {
        self.dispatcher.with(system);
        self
    }

    /// signals the dispatcher to call the on_start functions
    pub fn on_start(&mut self, resources : &mut Resources) {
        self.dispatcher.on_start(resources);
    }

    /// signals the dispatcher to call the on_exit functions
    pub fn on_exit(&mut self, resources : &mut Resources) {
        self.dispatcher.on_exit(resources);
    }

    /// signals the dispatcher to call the on_pause functions
    pub fn on_pause(&mut self, resources : &mut Resources) {
        self.dispatcher.on_pause(resources);
    }

    /// signals the dispatcher to call the on_resume functions
    pub fn on_resume(&mut self, resources : &mut Resources) {
        self.dispatcher.on_resume(resources);
    }

    /// signals the dispatcher to call the on_update functions
    pub fn on_update(&mut self, resources : &mut Resources) -> Trans {
       self.dispatcher.on_update(resources)
    }
}