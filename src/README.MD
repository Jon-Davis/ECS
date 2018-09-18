# State Entity Component System

This is an Entity Component System writen in rust that uses a simple State Machine to handle
which systems will be run.

## State Machine

The core loop happens inside of a State Machine. The State Machine is simply a stack of 
State structs. When the state machine is run, it will peek into the stack and run one iteration of the top
State. After each iteration, the State will return a Transition. There are 4 supported transitions.

*Transition::None will be no transition, which means that on the next iteration, the State Machine Will call
the on_update function for the same State.
*Transition::Pop will pop the current state off of the Stack. The on_exit functions of the State will be called,
then if there is a State below it, the on_resume function of the new state will run, followed by on_update.
*Transition::Swap(state) will pop the current state off of the State Machine and addthe new state onto the Stack,
The on_exit function of the current state will be called, followed by the on_start function of the new state.
*Transition::Push(stack) will push a new state ontop of the stack. The on_pause function of the current state
will be called, followed by the on_start function of the new State.

Additionally the State Machine will contain a Resources struct which will house the Entities and Components.

## State

States are a collection of Systems that are run when given events occur. There are 5 main events:
*on_update Called once per iteration of the State Machine
*on_start Called once the State becomes the top of the State Machines Stack
*on_exit Called once the State is removed from the State Machine
*on_pause Called when a State is added on top of the current state
*on_resume Called when the State above it was poped.

States contain a dispatcher. When a system is registered to a State, it is added to the dispatcher
which decides when that system will be called.

```rust
let intial_state = State::new()
    .with(System::new().set_start(&start_up).set_update(&hello_world))
    .with(System::new().set_update(&good_bye));
let mut sm = StateMachine::new(intial_state);
sm.run();
```

The above code creates an intial_state that the State Machine will have. This state has 2 systems associated with it.
The first system has an on_start function and an on_update function. The second system only has an on_update function.
Once the State Machine is created and the run function is called. The intial state will run a single iteration
every time the State Machine calls it's update function. The Initial State is responsible for handling how it's systems are
executed, but this one will Execute it's systems in FIFO order. 

## Systems

Systems are the functions associated with the ECS system, they are a struct of functions that take in a refrence to a Resources struct.
Every system can respond to any of the 5 events stated above.

```rust
fn hello_world(_ : &mut Resources) -> Trans {
    println!("Hello World!");
    Trans::None
}

...

let hello_world_system = System::new().set_update(&hello_world);
```

The above code looks just like a function, that is because it is! The hello_world function takes in a reference to some Resources and returns
a Transition. This simple function prints hello world, then performs a None Transition. If we were to create a state with just this system.
It would print Hello World! forever! 

## Entities and Components

If Systems are the functional portions of the ECS, then the Entities and Componets are the data. An Analogy to Objects
is that Entities are Objects and Components are Fields. We can Create an Entity with a position and a sprite and thats
similar to creating an object with a position and sprite. The Advantage is we can Dynamically create Entities with varying
amounts of Components at run time. We can also have our system work at the Compoent level, rather than the Object level. These
Components are stored in the Resources object, and can be created and accessed by other systems.

```rust
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

...

let hello_world_system = System::new().set_start(&start_up).set_update(&hello_world)
```

The above code is the entire system shown earlier. Here the start_up function creates two Components: A and B. Both of these structs impl the Componet trait.
(Additionaly they derive copy, but thats not required). It then creates three Entities, two with a Componet A and one with both a Componet A and Componet B.
Entities and Componets can be removed and accessed in simalar ways, and it doesn't have to be from the system that originally created it.