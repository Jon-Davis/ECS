use state::{Trans, State};
use resources::Resources;

/// Returns the update Status of the StateMachine
pub enum UpdateStatus {
    Continue,
    Exit,
}

/// A state machine contains a Stack of States 
/// And a package of Resources
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