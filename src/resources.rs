use component::Component;
use std::collections::HashMap;
use std::ops::Range;
use std::any::{TypeId, Any};
use entity::{Entity,EntityRegister};
use bit_field::BitField;
use std::mem::transmute;

const ENTITY_BITS : Range<usize> = 0..36;
const NEXT_BITS : Range<usize> = 36..63;
const IS_ON_BIT : Range<usize> = 63..64;

/*************************************************/
/* Trait of a Homogenous Collection of Components*/
/*************************************************/
trait ComponentCollection  {
    fn push(&mut self, component : Box<Any>, entity_id : u64) -> Result<(),Box<Any>>;
    fn remove(&mut self, entity_id : u64);
    fn get_id(&self) -> TypeId;
    fn len(&self) -> usize;
}

/*************************************************/
/* Wrapper which stores Components and Meta Data */
/*************************************************/
struct ComponentWrapper<D : Component> {
    component : D,
    meta : u64,
}

impl<D : Component> ComponentWrapper<D> {
    /// Constructs a new Component Wrapper, and initializes meta data
    fn new(component: Box<D>, entity_id : u64, next : u64, active : bool) -> ComponentWrapper<D> {
        ComponentWrapper {
            component: *component,
            meta: *0.set_bits(ENTITY_BITS, entity_id)
                    .set_bits(NEXT_BITS, next)
                    .set_bits(IS_ON_BIT, active as u64),
        }
    }

    /// Sets the next field of the meta data
    fn set_next(&mut self, next : u64){
        self.meta.set_bits(NEXT_BITS, next);
    }

    /// Sets the next field of the meta data
    fn get_next(&self) -> u64 {
        self.meta.get_bits(NEXT_BITS)
    }

    fn get_entity(&self) -> u64 {
        self.meta.get_bits(ENTITY_BITS)
    }
}

/*************************************************/
/* Stores a Single type of Component             */
/*************************************************/
struct ComponentVector<D : Component> {
    components : Vec<ComponentWrapper<D>>,
    type_id: TypeId,
    head: usize,
    tail: usize,
}

impl<D : Component> ComponentVector<D> {
    fn new() -> ComponentVector<D> {
        ComponentVector {
            components : Vec::new(),
            type_id: TypeId::of::<D>(),
            head: 0,
            tail: 0,
        }
    }

    fn iter(&self) -> ComponentVectorIter<D> {
        ComponentVectorIter::new(self)
    }
}

impl<D : Component> ComponentCollection for ComponentVector<D> {
    /// Push a new component onto the ComponentCollection.
    /// Invaraint : The new entity_id is > all previous entity_ids
    fn push(&mut self, component : Box<Any>, entity_id : u64) -> Result<(),Box<Any>> {
        match component.downcast::<D>() {
            Ok(deref) => {
                // Check if this is the first Component to be added
                // to the collection
                if self.len() == 0 {
                    self.head = 0;
                    self.tail = 0;
                } else {
                    // Get the new index for the new component
                    let new_index = self.components.len();

                    // Update the previous tail's next to point to the new index
                    if self.len() > self.tail {
                        self.components[self.tail].set_next(new_index as u64);
                    }

                    // update the tail to point to the new index
                    self.tail = new_index;
                }

                // Insert the new component into the vector
                Ok(self.components.push(ComponentWrapper::new(deref,entity_id,0,true)))
            },
            Err(e) => Err(e),
        }
    }

    fn remove(&mut self, entity_id : u64) {
        // Can't remove from empty vector
        if self.len() == 0 {
            return;
        }

        let mut prev: isize = -1;
        let mut curr: isize = self.head as isize;
        let mut found = false;
        let mut end_prev = self.head as isize;
        let mut found_end_prev = false;

        // chances our end_prev = len() -2 so let's checkot
        if self.len() >= 2 {
            if self.components[self.len() - 2].get_next() == (self.len() - 1) as u64{
                found_end_prev = true;
                end_prev = (self.len() - 2) as isize;
            } 
        }

        // Iterate over the structure till curr is found
        // keep a reference to the previous
        for value in self.iter(){
            // Finds the entity being removed
            if value.get_entity() == entity_id{
                found = true;
            } else if !found {
                prev = curr;
                curr = value.get_next() as isize;
            }

            // finds the entity that points to the last node
            if value.get_next() == (self.len() - 1) as u64{
                found_end_prev = true;
            } else if !found_end_prev {
                end_prev = value.get_next() as isize;
            }

            if found && found_end_prev {
                break;
            }
        }

        if found {
            // Since we will be swaping the end of the vector with the
            // currently deleted, we need to update the next pointer of the
            // prev entry to reflect that.
            if found_end_prev {
                self.components[end_prev as usize].set_next(curr as u64);
            }

            let curr = curr as usize;
            let next = self.components[curr as usize].get_next();

            // update the prev pointer
            if prev >= 0 {
                self.components[prev as usize].set_next(next);
            }

            // If curr was the head of the list, set the new head
            if curr == self.head {
                self.head = next as usize;
            }

            // If curr was the tail of the list, set the tail pointer
            if curr == self.tail {
                self.tail = prev.max(0) as usize;
            }

            if self.tail == self.len() - 1 {
                self.tail = curr;
            }

            self.components.swap_remove(curr);
        }
    }

    fn get_id(&self) -> TypeId {
        self.type_id
    }

    fn len(&self) -> usize {
        self.components.len()
    }
}

/*************************************************/
/* Iterator for Component Vector                 */
/*************************************************/
struct ComponentVectorIter<'a, D: Component> {
    current : usize,
    len : usize,
    data: &'a ComponentVector<D>,
}

impl<'a, D: Component> ComponentVectorIter<'a, D> {
    fn new(vector : &ComponentVector<D>) -> ComponentVectorIter<D> {
        ComponentVectorIter {
            current : vector.head,
            len : 0,
            data: &vector, 
        }
    }
}

impl<'a, D: Component> Iterator for ComponentVectorIter<'a, D>{
    type Item = &'a ComponentWrapper<D>;

    fn next(&mut self) -> Option<&'a ComponentWrapper<D>> {
        if self.len >= self.data.len() || self.current >= self.data.len() {
            None
        } else {
            match self.data.components.get(self.current) {
                Some(value) => {
                    self.len += 1;
                    self.current = value.get_next() as usize;
                    Some(&value)
                },
                None => None,
            }
        }
    }
}

pub struct CompVecIter<'a, D : Component> {
    iter: ComponentVectorIter<'a, D>,
}

impl<'a, D : Component> CompVecIter<'a, D> {
    fn new(vector : &'a ComponentVector<D>) -> CompVecIter<D> {
        CompVecIter {
            iter : ComponentVectorIter::new(vector), 
        }
    }
}

impl<'a, D: Component> Iterator for CompVecIter<'a, D>{
    type Item = &'a D;

    fn next(&mut self) -> Option<&'a D> {
        match self.iter.next(){
            Some(d) => Some(&d.component),
            None => None,
        }
    }
}

/*************************************************/
/* Stores a Collection of ComponentCollections   */
/*************************************************/
pub struct Resources {
    component_collections : HashMap<TypeId, Box<ComponentCollection>>,
    register: EntityRegister,
}

impl Resources {
    /// creates a new Resources struct with it's own Resource Managers and
    /// EntityRegister
    pub fn new() -> Resources {
        Resources {
            component_collections: HashMap::new(),
            register: EntityRegister::new(),
        }
    }

    /// Adds a component to an entity
    pub fn add<T : Component>(&mut self, component : T, entity : u64) -> Result<(), Box<Any>> {
        let id = TypeId::of::<T>();

        // check to make sure the component being removed exists, if not add it
        if !self.component_collections.contains_key(&id){
            let new_vector : ComponentVector<T> = ComponentVector::new();
            self.component_collections.insert(id, Box::new(new_vector));
        } 

        let manager = self.component_collections.get_mut(&id).expect("Resource not found");
        manager.push(Box::new(component), entity)
        
    }

    /// Removes a component from an entity
    pub fn remove<T : Component>(&mut self, entity : u64){
        let id = TypeId::of::<T>();

        // Check to make sure that a component exists before removing it
        match self.component_collections.get_mut(&id) {
            Some(manager) => manager.remove(entity),
            None => ()
        }
    }

    /// Creates a new entity in the resources
    pub fn new_entity(&mut self) -> Entity {
        let num = self.register.register(1);
        Entity {
            id: num.start,
            model : self,
        }
    }
    
    // Returns a reference to a componentvector with the given T
    fn get_component_vector<T:Component>(&mut self) -> Option<&ComponentVector<T>> {
        let id = TypeId::of::<T>();
        match self.component_collections.get(&id) {
            Some(cc) => {
                // NOTE: use of unsafe
                let col : Option<&ComponentVector<T>> = unsafe {
                    let c = transmute::<&Box<ComponentCollection>, &Box<ComponentVector<T>>>(cc);
                    if c.get_id() == id {
                        Some(&*c)
                    } else {
                        None
                    }
                };
                col
            }
            None => None,
        }
    }

    /// Gets an entire list of Components
    pub fn get<T : Component>(&mut self) -> Option<CompVecIter<T>>{
        match self.get_component_vector::<T>() {
            Some(vec) => {
                Some(CompVecIter::new(vec))
            },
            None => None,
        }
    }
}

/*************************************************/
/* Unit Tests                                    */
/*************************************************/
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    struct CompA {
        id: u64
    }
    struct CompB {
        id: u64
    }

    impl CompA{
        fn new(id : u64) -> CompA {
            CompA {
                id: id,
            }
        }
    }
    impl CompB{
        fn new(id : u64) -> CompB {
            CompB {
                id: id,
            }
        }
    }

    impl Component for CompA{}
    impl Component for CompB{}

    #[test]
    fn test_basic_component_vector() {
        let a = CompA::new(0);
        let b = CompA::new(1);
        let c = CompA::new(2);
        let mut cv : ComponentVector<CompA> = ComponentVector::new();
        let _ = cv.push(Box::new(a), 0);
        let _ = cv.push(Box::new(b), 1);
        let _ = cv.push(Box::new(c), 2);
        let mut i = 0;
        for item in cv.iter() {
            assert!(item.get_entity() == item.component.id);
            i += 1;
        }
        assert!(i == 3);
        cv.remove(1);
        i = 0;
        for item in cv.iter(){
            assert!(item.get_entity() == item.component.id);
            i+=1;
        }
        assert!(i == 2);
    }

    #[test]
    fn test_adv_component_vector(){
        let a = CompA::new(0);
        let b = CompA::new(1);
        let c = CompA::new(2);
        let d = CompA::new(3);
        let e = CompA::new(4);
        let f = CompA::new(5);
        let g = CompA::new(6);
        let mut cv : ComponentVector<CompA> = ComponentVector::new();
        let _ = cv.push(Box::new(a), 0);
        let _ = cv.push(Box::new(b), 1);
        let _ = cv.push(Box::new(c), 2);
        let _ = cv.push(Box::new(d), 3);
        let _ = cv.push(Box::new(e), 4);
        let _ = cv.push(Box::new(f), 5);

        // Insert 5 elements
        let actual : Vec<u64> = [0, 1, 2, 3, 4, 5].iter().map(|d| *d as u64).collect();
        let order : Vec<u64>  = [0, 1, 2, 3, 4, 5].iter().map(|d| *d as u64).collect();
        let cv_order : Vec<u64> = cv.iter().map(|c| c.get_entity()).collect();
        let cv_actual : Vec<u64> = cv.components.iter().map(|c| c.get_entity()).collect();
        assert!(actual == cv_actual);
        assert!(order == cv_order);

        // remove element 2
        cv.remove(2);
        let actual : Vec<u64> = [0, 1, 5, 3, 4].iter().map(|d| *d as u64).collect();
        let order : Vec<u64>  = [0, 1, 3, 4, 5].iter().map(|d| *d as u64).collect();
        let cv_order : Vec<u64> = cv.iter().map(|c| c.get_entity()).collect();
        let cv_actual : Vec<u64> = cv.components.iter().map(|c| c.get_entity()).collect();
        assert!(actual == cv_actual);
        assert!(order == cv_order);

        // remove element 4
        cv.remove(4);
        let actual : Vec<u64> = [0, 1, 5, 3].iter().map(|d| *d as u64).collect();
        let order : Vec<u64>  = [0, 1, 3, 5].iter().map(|d| *d as u64).collect();
        let cv_order : Vec<u64> = cv.iter().map(|c| c.get_entity()).collect();
        let cv_actual : Vec<u64> = cv.components.iter().map(|c| c.get_entity()).collect();
        assert!(actual == cv_actual);
        assert!(order == cv_order);

        // remove element 0
        cv.remove(0);
        let actual : Vec<u64> = [3, 1, 5].iter().map(|d| *d as u64).collect();
        let order : Vec<u64>  = [1, 3, 5].iter().map(|d| *d as u64).collect();
        let cv_order : Vec<u64> = cv.iter().map(|c| c.get_entity()).collect();
        let cv_actual : Vec<u64> = cv.components.iter().map(|c| c.get_entity()).collect();
        assert!(actual == cv_actual);
        assert!(order == cv_order);

        // add element 6
        let _ = cv.push(Box::new(g), 6);
        let actual : Vec<u64> = [3, 1, 5, 6].iter().map(|d| *d as u64).collect();
        let order : Vec<u64>  = [1, 3, 5, 6].iter().map(|d| *d as u64).collect();
        let cv_order : Vec<u64> = cv.iter().map(|c| c.get_entity()).collect();
        let cv_actual : Vec<u64> = cv.components.iter().map(|c| c.get_entity()).collect();
        assert!(actual == cv_actual);
        assert!(order == cv_order);
    }

    #[test]
    fn test_res(){
        let mut res = Resources::new();
        res.new_entity().with::<CompA>(CompA::new(0));
        res.new_entity().with::<CompA>(CompA::new(1)).with::<CompB>(CompB::new(5));
        res.new_entity().with::<CompA>(CompA::new(2));
        // Iterate through the A Components
        match res.get::<CompA>() {
            Some(col)=> {
                let mut i = 0;
                for a in col {
                    assert!(a.id == i);
                    i += 1;
                }
                assert!(i == 3);
            },
            None => assert!(false, "Get did not return an iterator"),
        }
        // Iterate through the B Components
        match res.get::<CompB>() {
            Some(col) => {
                let mut i = 0;
                for b in col {
                    assert!(b.id == 5);
                    i += 1;
                }
                assert!(i == 1);
            },
            None => assert!(false, "Get did not return an iterator"),
        }
    }
}