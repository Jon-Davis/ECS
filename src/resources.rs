use std::any::{TypeId};
use std::collections::HashMap;
use component::Component;
use std::ops::Range;
use bit_field::BitField;
use entity::{Entity,EntityRegister};

pub struct Resource {
    meta: u64,
    resource: Box<Component>,
}

const ENTITY_BITS : Range<usize> = 0..36;
const NEXT_BITS : Range<usize> = 36..63;
const IS_ON_BIT : Range<usize> = 63..64;

impl Resource {
    /// Creates a new Resource that wraps a Component
    fn new(component : Box<Component>, entity : u64) -> Resource {
        let mut meta = 0;
        meta.set_bits(IS_ON_BIT, 1);
        meta.set_bits(ENTITY_BITS, entity);
        meta.set_bits(NEXT_BITS, 0);
        Resource {
            meta : meta,
            resource : component,
        }
    }

    /// This function returns the associated entity
    /// id number for this component
    fn get_entity(&self) -> u64 {
        self.meta.get_bits(ENTITY_BITS)
    }

    /// This function gets the index of the next Resource
    /// in the Resource manager
    fn get_next(&self) -> u64 {
        self.meta.get_bits(NEXT_BITS)
    }

    /// This function sets the index of the next Resource
    /// in the Resource manager
    fn set_next(&mut self, next : u64){
        self.meta.set_bits(NEXT_BITS, next);
    }

    /// Retrieve meta data for if the Component is active
    /// or not. This does not mean the Component is freed
    /// This will simply be used if the Component wants to
    /// disable itself without deleting
    fn get_active(&self) -> u64{
        self.meta.get_bits(IS_ON_BIT) 
    }
}

/// The ResourceManager manages a single type of Component
/// and uses Generational Indexes to limit the size
pub struct ResourceManager {
    resources: Vec<Resource>,
    freed: Vec<usize>,
    tail: isize,
    head: usize,
}

/// Since ResourceManager isn't a simple datastructure
/// The ResourceManagerIter can be used to iterate through
/// The ResourceManager
pub struct ResourceManagerIter<'a> {
    manager : &'a ResourceManager,
    curr: isize,
}

impl<'a> Iterator for ResourceManagerIter<'a> {
    type Item = &'a Resource;

    /// Returns the next resource in the Iter
    fn next(&mut self) -> Option<&'a Resource>{
        if self.curr == -1 || self.manager.len() == 0 {
            None
        } else {
            let index = self.curr as usize;
            let value = &self.manager.resources[index];
            self.curr = if self.curr == self.manager.tail {
                -1
            } else {
                value.get_next() as isize
            };
            Some(value)
        }
    }
}

impl ResourceManager {
    /// Creates a new ResourceManager that will handle a 
    /// single type of Resource
    pub fn new() -> ResourceManager {
        ResourceManager{
            resources: Vec::new(),
            freed: Vec::new(),
            tail: -1,
            head: 0,
        }
    }

    /// This function adds a resource into the ResourceManager
    pub fn add(&mut self, c : Resource) {
        // Base case if this is the first resource
        if self.tail == -1 && self.freed.len() == 0 {
            self.resources.push(c);
            self.tail = 0;
            self.head = 0;
            return
        }

        // Get the index of the new Resource
        let new_index = if self.freed.len() == 0 {
            self.resources.push(c);
            self.resources.len() - 1
        } else {
            let i = self.freed.pop().unwrap();
            self.resources[i] = c;
            i
        };

        // Set the tail's next pointer to the new index
        self.resources[self.tail as usize].set_next(new_index as u64);
        self.tail = new_index as isize;
    }

    /// Returns a ResourceManagerIter which can be used to iterate
    /// over the components that are being managed
    pub fn iter(&self) -> ResourceManagerIter {
        ResourceManagerIter {
            curr : self.head as isize,
            manager : self,
        }
    }

    /// Returns the current len of the ResourceManager
    /// This is the length of all the components allocated
    /// minus all of the componets freed, giving the total
    /// number of componets being managed, not the total size
    /// allocated.
    pub fn len(&self) -> usize {
        self.resources.len() - self.freed.len()
    }

    /// Removes a given component that this ResourceManager manages
    /// from a specified entity id
    pub fn remove(&mut self, entity : u64){
        let mut prev : isize = -1;
        let mut curr : isize = -1;

        // Loop through the componenets and find the one
        // with the current entity, keeping track of curr and prev
        for res in self.iter(){
            if res.get_entity() == entity {
                if curr == -1 {
                    curr = 0;
                }
                break;
            } else {
                prev = curr;
                curr = res.get_next() as isize;
            }
        }

        // If the component was in the manager add it to the freed slot
        // If there was a previous one, update it to the next one
        if curr >= 0 {
            self.freed.push(curr as usize);
            let next = self.resources[curr as usize].get_next();

            if prev >= 0 {
                self.resources[prev as usize].set_next(next);
            }

            if curr as usize == self.head && self.len() > 0 {
                self.head = next as usize;
            }

            if curr == self.tail {
                self.tail = prev;
            }
        }
    }
}

/// The Resources struct contains all the resources of a state machine
/// It also contains an Enitity register so that all entities can have a
/// unique id number
pub struct Resources {
    components : HashMap<TypeId, ResourceManager>,
    pub register: EntityRegister,
}

impl Resources {
    /// creates a new Resources struct with it's own Resource Managers and
    /// EntityRegister
    pub fn new() -> Resources {
        Resources {
            components: HashMap::new(),
            register: EntityRegister::new(),
        }
    }

    /// Adds a component to an entity
    pub fn add<T : Component>(&mut self, component : T, entity : u64) {
        let id = TypeId::of::<T>();

        // check to make sure the component being removed exists, if not add it
        if !self.components.contains_key(&id){
            self.components.insert(id, ResourceManager::new());
        } 

        let manager = self.components.get_mut(&id).expect("Resource not found");
        let new_resource = Resource::new(Box::new(component),entity);
        manager.add(new_resource);
    }

    /// Removes a component from an entity
    pub fn remove<T : Component>(&mut self, entity : u64){
        let id = TypeId::of::<T>();

        // Check to make sure that a component exists before removing it
        match self.components.get_mut(&id) {
            Some(manager) => manager.remove(entity),
            None => ()
        }
    }

    /// Creates a new entity in the resources
    pub fn new_entity(&mut self) -> Entity {
        Entity::new(self)
    }

    /// Gets an entire list of Components
    pub fn get<T : Component>(&mut self) -> Option<&mut ResourceManager>{
        let id = TypeId::of::<T>();
        self.components.get_mut(&id)
    }
}
