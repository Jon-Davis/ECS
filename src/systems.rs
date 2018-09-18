use state::Trans;
use resources::Resources;

/// A system is a series of functions that can be called at certain times
pub struct System {
    pub on_update: &'static Fn(&mut Resources) -> Trans,
    pub on_exit: &'static Fn(&mut Resources),
    pub on_resume: &'static Fn(&mut Resources),
    pub on_pause: &'static Fn(&mut Resources),
    pub on_start: &'static Fn(&mut Resources),
}

impl System {
    /// Creates a new system
    pub fn new() -> System {
        System {
            on_update:  &|_ : &mut Resources| Trans::None,
            on_exit: &|_ : &mut Resources| (),
            on_resume: &|_ : &mut Resources| (),
            on_pause: &|_ : &mut Resources| (),
            on_start: &|_ : &mut Resources| (),
        }
    }

    /// change the on_update function of the system
    pub fn set_update(mut self, func : &'static Fn(&mut Resources) -> Trans) -> Self{
        self.on_update = func;
        self
    }

    /// change the on_start function of the system
    pub fn set_start(mut self, func : &'static Fn(&mut Resources)) -> Self{
        self.on_start = func;
        self
    }

    /// calls the on_update function of the system
    pub fn on_update(&self, resources : &mut Resources) -> Trans {
        let update = self.on_update;
        update(resources)
    }

    /// calls the on_exit function of the system
    pub fn on_exit(&self, resources : &mut Resources) {
        let exit = self.on_exit;
        exit(resources);
    }

    /// calls the on_pause function of the system
    pub fn on_pause(&self, resources : &mut Resources) {
        let pause = self.on_pause;
        pause(resources);
    }

    /// calls the on_start function of the system
    pub fn on_start(&self, resources : &mut Resources) {
        let start = self.on_start;
        start(resources);
    }

    /// calls the on_resume function of the system
    pub fn on_resume(&self, resources : &mut Resources) {
        let resume = self.on_resume;
        resume(resources);
    }
}