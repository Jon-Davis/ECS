extern crate bit_field;

mod state;
mod systems;
mod dispatcher;
mod resources;
mod entity;

use resources::{Resources,Component};
use systems::System;
use state::{State, StateMachine, Trans};

#[derive(Debug, Copy, Clone)]
struct CompA(u64);
#[derive(Debug, Copy, Clone)]
struct CompB(u64);
impl Component for CompA {}
impl Component for CompB {}

struct SystemA {}
struct SystemB {}
struct SystemC {}

impl System for SystemA {
    fn start(&mut self, res : &mut Resources) {
        let a = CompA(0);
        let b = CompB(1);
        res.new_entity().with::<CompA>(a);
        res.new_entity().with::<CompA>(a).with::<CompB>(b);
        res.new_entity().with::<CompA>(a);
    }

    fn update(&mut self, _ : &mut Resources) -> Trans {
        println!("Hello World!");
        Trans::None
    }
}

impl System for SystemB {
    fn update(&mut self, res : &mut Resources) -> Trans {
        println!("Good Bye!");
        res.remove::<CompA>(2);
        let next_state = State::new()
            .with(Box::new(SystemC{}));
        Trans::Swap(next_state)
    }
}

impl System for SystemC {
    fn update(&mut self, res : &mut Resources) -> Trans {
        println!("Hello 2!");
        {
            let c = res.get::<CompA>();
            match c {
                Some(v) => {
                    let aes : Vec<&CompA> = v.collect();
                    println!("{}", aes.len());
                }
                None => println!("Nothing found")
            };
        }
        {
            let b = res.get::<CompB>();
            match b {
                Some(v) => {
                    let bes : Vec<&CompB> = v.collect();
                    println!("{}", bes.len());
                },
                None => println!("Nothing found")
            };
        }
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
