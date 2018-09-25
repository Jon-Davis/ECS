use std::ops::Range;
use std::any::{TypeId, Any};
use entity::{Entity,EntityRegister};
use std::sync::Mutex;
use bit_field::BitField;
use std::cell::RefMut;
use std::mem::transmute;
use syncmap::{SyncMap,Request,Loan};

const ENTITY_BITS : Range<usize> = 0..36;
const NEXT_BITS : Range<usize> = 36..63;
const IS_ON_BIT : Range<usize> = 63..64;

/*************************************************/
/* Trait Stored in the ComponentCollections      */
/*************************************************/
pub trait Component: Any + Send + Sync {}

/*************************************************/
/* Trait of a Homogenous Collection of Components*/
/*************************************************/
pub trait ComponentCollection : Send + Sync {}

/*************************************************/
/* Wrapper which stores Components and Meta Data */
/*************************************************/
struct ComponentWrapper<D : Component> {
    component : D,
    meta : u64,
}

impl<D : Component> ComponentWrapper<D> {
    /// Constructs a new Component Wrapper, and initializes meta data
    fn new(component: D, entity_id : u64, next : u64, active : bool) -> ComponentWrapper<D> {
        ComponentWrapper {
            component: component,
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
pub struct ComponentVector<D : Component> {
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

    fn len(&self) -> usize {
        self.components.len()
    }

    pub(crate) fn push(&mut self, component : D, entity_id : u64){
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
        self.components.push(ComponentWrapper::new(component,entity_id,0,true))
    }

    fn remove(&mut self, entity_id : u64){
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
}

impl<D : Component> ComponentCollection for ComponentVector<D> {}

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
    component_collections : SyncMap<TypeId, Box<ComponentCollection>>,
    register: Mutex<EntityRegister>,
}

impl Resources {
    /// creates a new Resources struct with it's own Resource Managers and
    /// EntityRegister
    pub fn new() -> Resources {
        Resources {
            component_collections: SyncMap::new(),
            register: Mutex::new(EntityRegister::new()),
        }
    }

    pub fn register<T: Component>(&self){
        let vec : ComponentVector<T> = ComponentVector::new();
        let _ = self.component_collections.insert( TypeId::of::<T>(), Box::new(vec));
    }

    pub fn request(&self, request : &ResourceRequest) -> Loan<TypeId,Box<ComponentCollection>> {
        self.component_collections.request(&request.request).unwrap().unwrap()
    }

    pub(crate) fn get_token(&self) -> ResourceToken<'_>{
        ResourceToken::new(self)
    }
}

/*************************************************/
/* A Resource Token allows for a single loan     */
/*************************************************/
pub struct ResourceToken<'a> {
    loan : Option<Loan<'a,TypeId,Box<ComponentCollection>>>,
    resources : &'a Resources,
}

impl<'a> ResourceToken<'a>{
    pub fn new(res : &'a Resources) -> ResourceToken<'a> {
        ResourceToken {
            loan : None,
            resources : res,
        }
    }

    pub fn register<T : Component>(&self) {
        self.resources.register::<T>();
    }

    pub fn register_entity(&self) -> Entity{
        let id = self.resources.register.lock().unwrap().register(1);
        Entity::new_with_id(id.start)
    }

    pub fn request(self, request : &ResourceRequest) -> ResourceToken<'a> {
        let resources = self.resources;
        //unpack!(self, ResourceRequest, Resources);
        drop(self);
        ResourceToken {
            loan : Some(resources.request(request)),
            resources: resources,
        }
    }

    pub fn loan(&self) -> Option<&Loan<'a,TypeId,Box<ComponentCollection>>>{
        match &self.loan {
            Some(loan) => Some(&loan),
            None => None,
        }
    }

    pub fn unpack<C : Component>(&self) -> Option<&Box<ComponentVector<C>>> {
        match self.loan {
            Some(ref loan) => {
                let id = TypeId::of::<C>();
                match loan.read(&id) {
                    Some(value) => {
                        let val = unsafe {
                            transmute::<&Box<ComponentCollection>, &Box<ComponentVector<C>>>(value)
                        };
                        Some(val)
                    },
                    None => None,
                }
            },
            None => None,
        }
    }

    pub fn unpack_mut<C : Component>(&self) -> Option<RefMut<&mut Box<ComponentVector<C>>>> {
        match self.loan {
            Some(ref loan) => {
                let id = TypeId::of::<C>();
                match loan.write(&id) {
                    Some(value) => {
                        let val = unsafe {
                            transmute::<RefMut<&mut Box<ComponentCollection>>, RefMut<&mut Box<ComponentVector<C>>>>(value)
                        };
                        Some(val)
                    },
                    None => None,
                }
            },
            None => None,
        }
    }
}

/*************************************************/
/* Stores a Collection of Requests for resources */
/*************************************************/
pub struct ResourceRequest {
    request: Request<TypeId>,
}

// Wrapper for Request
impl ResourceRequest {
    pub fn new() -> ResourceRequest {
        ResourceRequest {
            request : Request::new(),
        }
    }

    pub fn read<T : Component>(&mut self) -> &mut Self {
        let id = TypeId::of::<T>();
        self.request.read(id);
        self
    }

    pub fn write<T : Component>(&mut self) -> &mut Self {
        let id = TypeId::of::<T>();
        self.request.write(id);
        self
    }
}

/*************************************************/
/* Unit Tests                                    */
/*************************************************/
#[cfg(test)]
mod tests {
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
        let _ = cv.push(a, 0);
        let _ = cv.push(b, 1);
        let _ = cv.push(c, 2);
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
        let _ = cv.push(a, 0);
        let _ = cv.push(b, 1);
        let _ = cv.push(c, 2);
        let _ = cv.push(d, 3);
        let _ = cv.push(e, 4);
        let _ = cv.push(f, 5);

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
        let _ = cv.push(g, 6);
        let actual : Vec<u64> = [3, 1, 5, 6].iter().map(|d| *d as u64).collect();
        let order : Vec<u64>  = [1, 3, 5, 6].iter().map(|d| *d as u64).collect();
        let cv_order : Vec<u64> = cv.iter().map(|c| c.get_entity()).collect();
        let cv_actual : Vec<u64> = cv.components.iter().map(|c| c.get_entity()).collect();
        assert!(actual == cv_actual);
        assert!(order == cv_order);
    }

    #[test]
    fn test_res(){
       
        
    }
}