use systems::System;
use state::Trans;
use resources::Resources;
use std::sync::Arc;
use rayon::prelude::*;

/// Responsible for deciding when systems get to
/// run, this simple Dispatcher executes systems 
/// in Fifo order
pub struct Dispatcher {
    systems : Vec<Box<System>>,
}

impl Dispatcher {
    /// Creates a new Dispatcher
    pub fn new() -> Dispatcher {
        Dispatcher {
            systems : Vec::new(),
        }
    }

    /// Adds a system to a dispatcher
    pub fn with(&mut self, system : Box<System>) -> &Self{
        self.systems.push(system);
        self
    }

    /// This will run the on_update function for all the systems that
    /// the dispatcher overlooks
    pub fn on_update(&mut self, resources : Arc<Resources>) -> Trans {
        self.systems.par_iter_mut().map(|system| {
            system.update(resources.get_token())
        }).reduce(|| Trans::None, |a,b| match a {
            Trans::None => b,
            _ => a,
        })
    }

    /// This will run the on_start function for all the systems that
    /// the dispatcher overlooks
    pub fn on_start(&mut self, resources : Arc<Resources>) {
        self.systems.par_iter_mut().for_each(|system| {
            system.start(resources.get_token());
        });
    }

    /// This will run the on_exit function for all the systems that
    /// the dispatcher overlooks
    pub fn on_exit(&mut self, resources : Arc<Resources>) {
        self.systems.par_iter_mut().for_each(|system| {
            system.exit(resources.get_token());
        });
    }

    /// This will run the on_pause function for all the systems that
    /// the dispatcher overlooks
    pub fn on_pause(&mut self, resources : Arc<Resources>) {
        self.systems.par_iter_mut().for_each(|system| {
            system.pause(resources.get_token());
        });
    }

    /// This will run the on_resume function for all the systems that
    /// the dispatcher overlooks
    pub fn on_resume(&mut self, resources : Arc<Resources>) {
        self.systems.par_iter_mut().for_each(|system| {
            system.resume(resources.get_token());
        });
    }
}