use state::Trans;
use resources::ResourceToken;

/// A system is a series of functions that can be called at certain times

pub trait System : Send + Sync {

        /// While this system is in the active the State 
        /// the update function will be called once per 'frame'
        fn update(&mut self, _res : ResourceToken) -> Trans {
            Trans::None
        }

        /// This function will be called only once before the
        /// first update of the system.
        fn start(&mut self, _res : ResourceToken) {
            
        }

        /// This function will be called only once when the 
        /// state that this system is bound to is released
        fn exit(&mut self, _res : ResourceToken) {

        }

        /// This function will be called whenever the current state
        /// is superseded by another state
        fn pause(&mut self, _res : ResourceToken){

        }

        /// This function sill be called whenever the current state
        /// is resumed from being superseded.
        fn resume(&mut self, _res : ResourceToken){

        }
}
