extern crate bit_field;

mod state;
mod systems;
mod dispatcher;
mod resources;
mod entity;

use resources::{Resources,Component};
use systems::System;
use state::{State, StateMachine, Trans};

struct CompA(u64);
struct CompB(u64);
impl Component for CompA {}
impl Component for CompB {}

struct SystemA {}
struct SystemB {}
struct SystemC {}

impl System for SystemA {
    fn start(&mut self, res : &mut Resources) {
        res.new_entity().with::<CompA>(CompA(0));
        res.new_entity().with::<CompA>(CompA(73)).with::<CompB>(CompB(19));
        res.new_entity().with::<CompB>(CompB(23));
    }

    fn update(&mut self, _ : &mut Resources) -> Trans {
        println!("Hello World!");
        Trans::None
    }
}

impl System for SystemB {
    fn update(&mut self, res : &mut Resources) -> Trans {
        println!("Good Bye!");
        res.remove::<CompB>(2);
        let next_state = State::new()
            .with(Box::new(SystemC{}));
        Trans::Swap(next_state)
    }
}

impl System for SystemC {
    fn update(&mut self, res : &mut Resources) -> Trans {
        match res.get::<CompA>() {
            Some(comp_a_iter) => {
                for comp_a in comp_a_iter {
                    println!("{}", comp_a.0);
                }
            }
            None => println!("Nothing found")
        };
        Trans::Pop
    }
}

fn main() {
    let intial_state = State::new()
        .with(Box::new(SystemA{}))
        .with(Box::new(SystemB{}));
    let mut sm = StateMachine::new(intial_state);
    sm.run();
}
