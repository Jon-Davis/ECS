use dispatcher::Dispatcher;
use systems::System;
use resources::Resources;

/*************************************************/
/* Valid State Transitions                       */
/*************************************************/
pub enum Trans {
    None,
    Pop,
    Push(State),
    Swap(State),
}

/*************************************************/
/* A State Struct handles the different systems  */
/*************************************************/
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
    pub fn with(mut self, system : Box<System>) -> State {
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

/// Returns the update Status of the StateMachine
pub enum UpdateStatus {
    Continue,
    Exit,
}

/*************************************************/
/* State Machine is a stack of State structs     */
/*************************************************/
pub struct StateMachine {
    stack: Vec<State>,
    resources: Resources,
}

impl StateMachine {
    /// Creates a new statemachine
    pub fn new(initial_state : State) -> StateMachine {
        StateMachine {
            stack: vec!(initial_state),
            resources: Resources::new(),
        }
    }

    /// Perfroms a single update on the StateMachine
    fn update(&mut self) -> UpdateStatus {
        match self.stack.len() {
            0 => UpdateStatus::Exit,
            _ => {
                match self.stack[0].on_update(&mut self.resources) {
                    Trans::None => UpdateStatus::Continue,
                    Trans::Pop => {
                        self.stack[0].on_exit(&mut self.resources);
                        self.stack.pop();
                        match self.stack.first_mut() {
                            Some(new_state) => {
                                new_state.on_resume(&mut self.resources);
                                UpdateStatus::Continue
                            }
                            None => UpdateStatus::Exit
                        } 
                    }
                    Trans::Push(mut new_state) => {
                        self.stack[0].on_pause(&mut self.resources);
                        new_state.on_start(&mut self.resources);
                        self.stack.push(new_state);
                        UpdateStatus::Continue
                    }
                    Trans::Swap(mut new_state) => {
                        self.stack[0].on_exit(&mut self.resources);
                        self.stack.pop();
                        new_state.on_start(&mut self.resources);
                        self.stack.push(new_state);
                        UpdateStatus::Continue
                    }
                }
            }
        }
    }

    /// Runs the StateMachine until it finishes
    pub fn run(&mut self) {
        if self.stack.len() > 0 {
            self.stack[0].on_start(&mut self.resources);
        }
        loop {
            match self.update() {
                UpdateStatus::Continue => continue,
                UpdateStatus::Exit => break,
            }
        } 
    }
}