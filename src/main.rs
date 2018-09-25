extern crate bit_field;
extern crate rayon;
mod syncmap;
mod state;
mod systems;
mod dispatcher;
mod resources;
mod entity;

use systems::System;
use resources::{Component, ResourceRequest, ResourceToken};
use state::{State, StateMachine, Trans};

struct CompInt(u32);
struct CompFloat(f32);
impl Component for CompInt{}
impl Component for CompFloat{}

struct SystemA {
    resources : ResourceRequest
}

impl System for SystemA {
    fn start(&mut self, token : ResourceToken) {
        // register components this system will use
        token.register::<CompInt>();
        token.register::<CompFloat>();

        // fill out the request on how components will be used
        self.resources.write::<CompInt>()
                      .write::<CompFloat>();

        // Lets get some resources and initialize defualt values
        let loan_token = token.request(&self.resources);

        // once the resources are retrieved, we will need to unpack them
        let ints = loan_token.unpack_mut::<CompInt>().unwrap();
        let float = loan_token.unpack_mut::<CompFloat>().unwrap();

        // register a new entity with the components
        loan_token.register_entity()
                .with(CompInt(0), ints)
                .with(CompFloat(5.5), float);
    }

    fn update(&mut self, token : ResourceToken) -> Trans {
        Trans::Pop
    }
}

struct SystemB {

}
impl System for SystemB{}

fn main() {
    let system_a = SystemA{
        resources : ResourceRequest::new()
    }; 
    let system_b = SystemB{}; // SystemB is an empty struct with an update function
    let intial_state = State::new()
        .with(Box::new(system_a))
        .with(Box::new(system_b));
    let mut sm = StateMachine::new(intial_state);
    sm.run();
}
