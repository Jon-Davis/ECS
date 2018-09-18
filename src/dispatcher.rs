use systems::System;
use state::Trans;
use resources::Resources;

/// Responsible for deciding when systems get to
/// run, this simple Dispatcher executes systems 
/// in Fifo order
pub struct Dispatcher {
    systems : Vec<System>,
}

impl Dispatcher {
    /// Creates a new Dispatcher
    pub fn new() -> Dispatcher {
        Dispatcher {
            systems : Vec::new(),
        }
    }

    /// Adds a system to a dispatcher
    pub fn with(&mut self, system : System) -> &Self{
        self.systems.push(system);
        self
    }

    /// This will run the on_update function for all the systems that
    /// the dispatcher overlooks
    pub fn on_update(&mut self, resources : &mut Resources) -> Trans {
        for system in self.systems.iter_mut() {
            match system.on_update(resources){
                Trans::None => continue,
                transition => return transition,
            }
        }
        Trans::None
    }

    /// This will run the on_start function for all the systems that
    /// the dispatcher overlooks
    pub fn on_start(&mut self, resources : &mut Resources) {
        for system in self.systems.iter_mut() {
            system.on_start(resources);
        }
    }

    /// This will run the on_exit function for all the systems that
    /// the dispatcher overlooks
    pub fn on_exit(&mut self, resources : &mut Resources) {
        for system in self.systems.iter_mut() {
            system.on_exit(resources);
        }
    }

    /// This will run the on_pause function for all the systems that
    /// the dispatcher overlooks
    pub fn on_pause(&mut self, resources : &mut Resources) {
        for system in self.systems.iter_mut() {
            system.on_pause(resources);
        }
    }

    /// This will run the on_resume function for all the systems that
    /// the dispatcher overlooks
    pub fn on_resume(&mut self, resources : &mut Resources) {
        for system in self.systems.iter_mut() {
            system.on_resume(resources);
        }
    }
}