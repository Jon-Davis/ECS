use std::collections::HashMap;
use std::sync::{Mutex, Condvar, PoisonError, MutexGuard};
use std::hash::Hash;
use std::{fmt,error};
use std::error::Error;
use std::cell::{RefCell,RefMut};
use std::fmt::{Display};

/************************************************************/
/* States whether a request is for Read or Write Permisions */
/************************************************************/
enum RequestType {
    Read,
    Write,
}

/*************************************************************/
/* A Request stores a list of requested keys and the         */
/* Read or Write Permision of the requested key              */
/*************************************************************/
pub struct Request<K : Eq + Hash> {
    resources : Vec<(K,RequestType)>,
}

impl<K : Eq + Hash> Request<K>{
    /// Constructs a new Request
    pub fn new() -> Request<K> {
        Request {
            resources : Vec::new(),
        }
    }

    /// Adds a key to the request, asking for read permisions
    pub fn read(&mut self, r : K) -> &mut Self {
        self.resources.push((r,RequestType::Read));
        self
    }

    /// Adds a key to the request, asking for write permisions
    pub fn write(&mut self, r : K) -> &mut Self {
        self.resources.push((r,RequestType::Write));
        self
    }
}

/************************************************************/
/* A Loaner represents a type that loans it's resources out */
/************************************************************/
trait Loaner<K : Eq + Hash,V> {
    fn resend(&self, &Loan<'_, K, V>);
}

/************************************************************/
/* A Loan stores loaned resources for reads and writes      */
/************************************************************/
pub struct Loan<'a, K : 'a + Eq + Hash, V : 'a> {
    reads: HashMap<K, &'a V>,
    writes: HashMap<K, RefCell<&'a mut V>>,
    owner: &'a Loaner<K, V>,
}

impl<'a, K : 'a + Eq + Hash, V : 'a> Loan<'a, K, V>{

    pub fn write(&self, key : &K) -> Option<RefMut<&'a mut V>>{
        match self.writes.get(key) {
            Some(value) => match value.try_borrow_mut() {
                Ok(val) => Some(val),
                _ => None,
            },
            None => None,
        }
    }

    pub fn read(&self, key : &K) -> Option<& V>{
        match self.reads.get(key) {
            Some(value) => Some(*value),
            None => None,
        }
    }
}

impl<'a, K : 'a + Eq + Hash, V : 'a> Drop for Loan<'a, K, V>{
    /// Resend the loan once it is droped
    fn drop(&mut self) {
        self.owner.resend(self);
    }
}

/************************************************************/
/* Contains the number of readers and writers of a value V  */
/************************************************************/
struct RwInfo<V> {
    readers : usize,
    writers : usize,
    value : V,
}

impl<V> RwInfo<V> {
    /// Constructs a new RwInfo around a value with defualts of 
    /// 0 readers and 0 writers
    fn new(value : V) -> RwInfo<V> {
        RwInfo {
            readers : 0,
            writers : 0,
            value : value,
        }
    }

    /// Returns true if there are no writers on the value
    fn can_read(&self) -> bool {
        self.writers == 0
    }

    /// Returns true if there are no readers or writers on
    /// the value
    fn can_write(&self) -> bool {
        self.writers == 0 && self.readers == 0
    }

    /// Increments the number of readers there are on the value
    fn read(&self, _guard : &MutexGuard<()>) -> &V {
        unsafe { 
            let s = (self as *const RwInfo<V> as *mut RwInfo<V>).as_mut().unwrap();
            s.readers+=1;
            &mut s.value
        }
    }

    /// Increments the number of writers there are on the value
    fn write(&self, _guard : &MutexGuard<()>) -> RefCell<&mut V> {
        unsafe { 
            let s = (self as *const RwInfo<V> as *mut RwInfo<V>).as_mut().unwrap();
            s.writers+=1;
            RefCell::new(&mut s.value)
        }
    }

    /// Decrements the number of readers there are on the value
    fn unread(&self, _guard : &MutexGuard<()>){
        unsafe { 
            let s = (self as *const RwInfo<V> as *mut RwInfo<V>).as_mut().unwrap();
            s.readers-=1;
        }
    }

    // Decrements the number of writers there are on the value
    fn unwrite(&self, _guard : &MutexGuard<()>) {
        unsafe { 
            let s = (self as *const RwInfo<V> as *mut RwInfo<V>).as_mut().unwrap();
            s.writers-=1;
        }
    }
}

/************************************************************/
/* A map that can loan out it's resources with RWLock       */
/************************************************************/
pub struct SyncMap<K : Eq + Hash, V> {
    map : RwInfo<HashMap<K, RwInfo<V>>>,
    mutex : Mutex<()>,
    condvar : Condvar,
}

impl<K : Eq + Hash + Clone, V> SyncMap<K, V> {
    /// Constructs a new Syncmap
    pub fn new() -> SyncMap<K, V> {
        SyncMap {
            map : RwInfo::new(HashMap::new()),
            mutex : Mutex::new(()),
            condvar : Condvar::new(),
        }
    }

    /// Unsafe function that returns a mutable refrence to self
    /// but requires that the mutex guard be passed in
    fn map_as_mut(&self, _guard : &MutexGuard<()>) -> &mut RwInfo<HashMap<K, RwInfo<V>>> {
        unsafe { (&self.map as *const RwInfo<HashMap<K, RwInfo<V>>> as *mut RwInfo<HashMap<K, RwInfo<V>>>).as_mut().unwrap() }
    }

    /// Inserts a new value into the SyncMap only if the key has not been entered
    /// before, does nothing if a key is already in the SyncMap
    pub fn insert(&self, key : K, value : V) -> Result<(),PoisionSyncMapError>{
        let mut guard = self.mutex.lock()?;
        loop {
            // First check to see if there is already a value at that location
            if self.map.can_read(){
                let map = self.map_as_mut(&guard);
                if let Some(_) = map.value.get(&key){
                    drop(guard);
                    self.condvar.notify_all();
                    return Ok(());
                }
            }
            // If the map doesnt already have a value initialize one
            if self.map.can_write() {
                let map = self.map_as_mut(&guard);
                map.value.insert(key, RwInfo::new(value));
                drop(guard);
                self.condvar.notify_all();
                return Ok(());
            }
            // Block if could not complete
            guard = self.condvar.wait(guard)?;
        }
    }

    /// Given a request of keys with read and write permisions
    /// Request will return refrences to the values. the values can have
    /// multiple readers at a time or 1 writer. If a request can not be fufilled
    /// it will block untill it can. Certain things such as a poision error or
    /// an invalid key will cause this function to return with either an error
    /// or with a None. Once it can fufill the request, it will return a Loan
    /// on the request.
    pub fn request(&self, request : &Request<K>) -> Result<Option<Loan<K,V>>,PoisionSyncMapError> {
        let mut guard = self.mutex.lock()?;
        loop {
            if self.map.can_read(){
                let map = self.map_as_mut(&guard);
                let mut available = true;

                // Check to see if all the requested resources are available
                for item in request.resources.iter(){
                    let (key,access) = item;
                    match (access,map.value.get(key)) {
                        // If the request is read, and the value can be read continue
                        (RequestType::Read, Some(value)) if value.can_read() => continue,
                        // If the request is write and the value can be writen continue
                        (RequestType::Write, Some(value)) if value.can_write() => continue,
                        // If the map does not contain a value, return from function
                        (_, None) => {
                            drop(guard);
                            self.condvar.notify_all();
                            return Ok(None)
                        },
                        // If the map contains the value but cant be read/writen, break from loop
                        _ => {available = false; break},
                    };
                }

                // aquire the resources if they are available
                if available {
                    // Create a read and write hashmap for the loan
                    let mut reads = HashMap::new();
                    let mut writes = HashMap::new();
                    // collect the loaned resources
                    for item in request.resources.iter(){
                        let (key,access) = item;
                        match (access,map.value.get(key)) {
                            (RequestType::Read, Some(value)) if value.can_read() => {
                                reads.insert((*key).clone(), value.read(&guard));
                            },
                            (RequestType::Write, Some(value)) if value.can_write() => {
                                writes.insert((*key).clone(), value.write(&guard));
                            },
                            _ => return Err(PoisionSyncMapError()),
                        };
                    }

                    // Signal there is a reader active on the SyncMap
                    map.read(&guard);
                    // Clean up and return the new Loan
                    drop(guard);
                    self.condvar.notify_all();
                    return Ok(Some(Loan {
                        reads: reads,
                        writes: writes,
                        owner: self,
                    }))
                }
            }
            // Block if could not complete
            guard = self.condvar.wait(guard)?;
        }
    }
}

impl<K : Eq + Hash + Clone,V> Loaner<K,V> for SyncMap<K,V> {
    /// Given a loan the SyncMap will revaluate the 
    /// RwInfos such that the resources are made available.
    fn resend(&self, loan : &Loan<'_, K, V>){
        let guard = self.mutex.lock().unwrap();
        let map = self.map_as_mut(&guard);
        for (reader,_) in loan.reads.iter() {
            map.value.get(reader).unwrap().unread(&guard);
        }
        for (writer,_) in loan.writes.iter() {
            map.value.get(writer).unwrap().unwrite(&guard);
        }
        map.unread(&guard);
        drop(guard);
        self.condvar.notify_all();
    }
}

/************************************************************/
/* Error type that wraps the PoisionError                   */
/************************************************************/
#[derive(Debug)]
pub struct PoisionSyncMapError();
impl Display for PoisionSyncMapError {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SyncMap has been poisioned")
    }
}
impl Error for PoisionSyncMapError {
    fn description(&self) -> &str {
        "SyncMap has been poisioned"
    }
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
impl<'a> From<PoisonError<MutexGuard<'a,()>>> for PoisionSyncMapError {
    fn from(_ : PoisonError<MutexGuard<'a,()>>) -> Self{
        PoisionSyncMapError()
    }    
}

/************************************************************/
/* SyncMap Tests                                            */
/************************************************************/
mod test {
    use super::*;
    use std::{thread,time};
    use std::sync::Arc;

    /// Test out if the sync map has basic functionality for
    /// reading and writing, withough introduction threads
    #[test]
    fn test_map(){
        let map = SyncMap::new();
        map.insert(0, vec!(0,1,2)).unwrap();
        map.insert(1, vec!(3,4,5)).unwrap();
        map.insert(2, vec!(6,7,8)).unwrap();

        let mut read_request = Request::new();
        read_request.read(0).read(1).read(2);

        let loan = map.request(&read_request).unwrap().unwrap();

        let mut i = 0;
        for key in 0..3 {
            for num in loan.read(&key).unwrap().iter(){
                assert!(*num == i, format!("num = {}, i = {}\n",num,i));
                i += 1;
            }
        }
        assert!(i == 9);
    }

    /// Test if the Sync map can handle multiple threads asking
    /// for read and write permisions.
    #[test]
    fn para_test(){
        // Initialize the syncmap
        let map = Arc::new(SyncMap::new());
        map.insert(0, vec!(0,1,2)).unwrap();
        map.insert(1, vec!(3,4,5)).unwrap();
        map.insert(2, vec!(6,7,8)).unwrap();

        // Request read permisions for 0 and 1 and write for 2
        let mut request = Request::new();
        request.read(0).read(1).write(2);

        // Make the request so that we have a loan, before making threads
        let loan = map.request(&request).unwrap().unwrap();

        // Spawn a thread to test if dual reading is possible
        let read_map = map.clone();
        let reader_thread = thread::spawn(move || {
            let mut read_request = Request::new();
            read_request.read(0).read(1);
            let read_loan = read_map.request(&read_request).unwrap().unwrap();
            let mut i = 0;
            for key in  0..2 {
                let vec = read_loan.read(&key).unwrap();
                for num in vec.iter(){
                    assert!(*num == i, format!("num = {}, i = {}\n",num,i));
                    i += 1;
                }
            }
            assert!(i == 6, format!("i = {}", i));
        });

        // Join on the reader thread to ensure dual reading has occured
        reader_thread.join().unwrap();

        // Now test to make sure exculisve write access is given
        let write_map = map.clone();
        let write_thread = thread::spawn(move || {
            // This thread should block here because 2 is already loaned with write
            let mut read_request = Request::new();
            read_request.read(2);
            let read_loan = write_map.request(&read_request).unwrap().unwrap();
            let vec = read_loan.read(&2).unwrap();
            assert!(vec.len() == 5);
            for i in vec.iter() {
                assert!(*i == 100);
            } 
        });

        // Sleep to make sure write_thread has a chance to run
        thread::sleep(time::Duration::from_millis(500));

        // update the values of the vec for key 2
        {
            let mut vec = loan.write(&2).unwrap();
            vec.push(0);
            vec.push(0);
            for i in vec.iter_mut() {
                *i = 100;
            }
        }

        // drop the loan to allow others to enter
        drop(loan);
        write_thread.join().unwrap();
    }

    #[test]
    fn multi_mut(){
        let map = Arc::new(SyncMap::new());
        map.insert(0, vec!(0,1,2)).unwrap();
        map.insert(1, vec!(3,4,5)).unwrap();

        // Request write permisions for 0 and 1
        let mut request = Request::new();
        request.write(0).write(1);

        // Make the request so that we have a loan
        let loan = map.request(&request).unwrap().unwrap();

        let mut zero = loan.write(&0).unwrap();
        let mut one = loan.write(&1).unwrap();
        // since zero is already borrowed return none
        let mut _zero_2 = match loan.write(&0) {
            Some(_) => assert!(false, "zero_2 write returned"),
            None => (),
        };

        for (z,o) in zero.iter_mut().zip(one.iter_mut()){
            let t = *z;
            *z = *o;
            *o = t;
        }

        let mut i = 0;
        for (z,o) in zero.iter().zip(one.iter()) {
            assert!(*z == i + 3,format!("z : {}, i + 3: {}",z,i+3));
            assert!(*o == i, format!("o : {}, i : {}",o,i));
            i += 1;
        }
    }
}