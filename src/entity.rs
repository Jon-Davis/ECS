use std::ops::Range;
use resources::Resources;
use component::Component;

/// The EntityRegister generates id's for 
/// Entities so that entities have unique names
pub struct EntityRegister {
    pub(crate) entity : u64,
}

impl EntityRegister {
    /// Creates a new EntityRegister
    pub fn new() -> EntityRegister {
        EntityRegister {
            entity : 0
        }
    }

    /// Registers new Entity id's, multiple id's can be registered
    /// at one time.
    pub fn register(&mut self, number_to_register : u64) -> Range<u64> {
        let start = self.entity;
        self.entity += number_to_register;
        start..self.entity
    }

}

/// An entity contains an ID and can be used to add
/// components to a Resource under it's id
pub struct Entity<'a>{
    pub(crate) id :  u64,
    pub(crate) model : &'a mut Resources,
}

impl<'a> Entity<'a> {
    /// Creates a new Entity given an Entity id, this does not consult the
    /// The register so use high values between 2^32..2^36 or use values
    /// previously registered with the Entity Register but not yet entered
    /// as Entities into the Resources section
    pub fn new_with_id(id : u64, res : &'a mut Resources) -> Entity<'a> {
        Entity {
            id: id,
            model : res,
        }
    }

    /// Returns the id of the Entity
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Add's a component to the resources under this entity
    pub fn with<T : Component>(self, comp : T) -> Self {
        self.model.add::<T>(comp,self.id);
        self
    }
}