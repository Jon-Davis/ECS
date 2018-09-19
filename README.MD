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
let system_a = SystemA{}; // SystemA is an empty struct with a start and update function
let system_b = SystemB{}; // SystemB is an empty struct with an update function
let intial_state = State::new()
    .with(Box::new(system_a))
    .with(Box::new(system_b));
let mut sm = StateMachine::new(intial_state);
sm.run();
```

The above code creates an intial_state that the State Machine will have. This state has 2 systems associated with it.
The first system has an on_start function and an on_update function. The second system only has an on_update function.
Once the State Machine is created and the run function is called. The intial state will run a single iteration
every time the State Machine calls it's update function. The Initial State is responsible for handling how it's systems are
executed, but this one will Execute it's systems in FIFO order. 

## Systems

Systems are the functions associated with the ECS system. System is a trait that implements 
start(), exit(), update(), pause(), and resume() functions. There is a defualt implementation
for each function. A system implementation should overide the functions that it wants to respond to.
```rust
impl System for SystemA {
    fn update(&mut self, _ : &mut Resources) -> Trans {
        println!("Hello World!");
        Trans::None
    }
}
```
This simple system above will be called once every 'frame' and print hello world. The function
will never cause a transition to occur, so if this is the only system in state, the state would
run forever.
All system functions take in 2 paramters, a mutable refrence to themselves, and a mutable
reference to the State Machines Resources.
The update function is special, becuase it returns a Transition. When special events occur, a system
can change the current state by returning a transition other than None. 

## Entities and Components

If Systems are the functional portions of the ECS, then the Entities and Componets are the data. 
An Analogy to Objects is that Entities are Objects and Components are Fields. 
We can Create an Entity with a position and a sprite and thats similar to creating an object with a position and sprite. 
The Advantage is we can Dynamically create Entities with varying amounts of Components at run time. We can also have our system work at the Compoent level, rather than the Object level. 
These Components are stored in the Resources object, and can be created and accessed by other systems.

```rust
struct CompA(u64);
struct CompB(u64);
impl Component for CompA {}
impl Component for CompB {}

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
```

The above code is the entire system shown earlier. Here the start_up function creates three entities.
The first entity has a single component of type CompA with a value of 0.
The second entity has two components, one of type CompA with a value of 73, and one of type CompB with a value of 19.
The third entity has a single component of type CompB with a value of 23.
Systems can create and use entities and components, however they can access resources created by other systems

```rust 
impl System for SystemB {
    fn update(&mut self, res : &mut Resources) -> Trans {
        println!("Good Bye!");
        res.remove::<CompB>(2);
        let next_state = State::new()
            .with(Box::new(SystemC{}));
        Trans::Swap(next_state)
    }
}
```

The above system was also attached into the initial state. This system will print Good Bye, then remove the
CompB associated with the entity 2 (the third entity created). Construct a new state with a system SystemC. 
Then swap the current state with the newly constructed state. After this sytem runs only the first and second
entity will be in the Resources.

```rust
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
```

Components can also be retireved from the Resources, for example you can get an iterator of all
the components of a certain type by calling the get method for a given type.
Above is the only system in the newly added state. It retrieves all of the CompA components in the resources
and prints out their value. This will result in 0 and 73.