extern crate bit_field;

mod state;
mod statemachine;
mod systems;
mod dispatcher;
mod resources;
mod component;
mod entity;

use resources::Resources;
use state::Trans;
use systems::System;
use statemachine::StateMachine;
use state::State;
use component::Component;

#[derive(Debug, Copy, Clone)]
struct CompA;
#[derive(Debug, Copy, Clone)]
struct CompB;
impl Component for CompA {}
impl Component for CompB {}

fn start_up(res : &mut Resources) {
    let a = CompA {};
    let b = CompB {};
    res.new_entity().with::<CompA>(a);
    res.new_entity().with::<CompA>(a).with::<CompB>(b);
    res.new_entity().with::<CompA>(a);
}

fn hello_world(_ : &mut Resources) -> Trans {
    println!("Hello World!");
    Trans::None
}

fn good_bye(res : &mut Resources) -> Trans {
    println!("Good Bye!");
    res.remove::<CompA>(2);
    let next_state = State::new()
        .with(System::new().set_update(&hello_two));
    Trans::Swap(next_state)
}

fn hello_two(res : &mut Resources) -> Trans {
    println!("Hello 2!");
    {
        let c = res.get::<CompA>();
        match c {
            Some(v) => println!("{}", v.len()),
            None => println!("Nothing found")
        };
    }
    {
        let b = res.get::<CompB>();
        match b {
            Some(v) => println!("{}", v.len()),
            None => println!("Nothing found")
        };
    }
    Trans::Pop
}

fn main() {
    let intial_state = State::new()
        .with(System::new().set_start(&start_up).set_update(&hello_world))
        .with(System::new().set_update(&good_bye));
    let mut sm = StateMachine::new(intial_state);
    sm.run();
}
