
use super::resources::Resources;
use super::params::{System, Fetch, ResMut, ResRef};
use super::handle::Resource;
use super::error::Trace;
use super::systems::Systems;
use super::archetypes::Archetypes;
use super::handle::Handle;
use super::handle::Component;

pub struct Ecs {
    pub(crate) archetypes: Archetypes,
    pub(crate) resources: Resources,
    pub(crate) systems: Systems,
}

impl Ecs {
    pub fn get_resource_mut<R: Resource>(&self) -> Trace<ResMut<R>> {
        self.resources.get_resource_mut::<R>()
    }

    pub fn get_resource_ref<R: Resource>(&self) -> Trace<ResRef<R>> {
        self.resources.get_resource_ref::<R>()
    } 

    pub fn add_component<C: Component>(&mut self) {
        static mut COUNTER: u16 = 0;

        unsafe { 
            *C::handle() = COUNTER; 
            COUNTER += 1;
        }
    }

    pub fn add_resource<R: Resource>(&mut self, resource: R) {
        self.resources.add_resource(resource)
    }

    pub fn add_startup<S: System + Fetch, H: Handle>(&mut self) {
        self.systems.add_startup::<S, H>();
    }

    pub fn add_system<S: System + Fetch, H: Handle>(&mut self) {
        self.systems.add_system::<S, H>();
    }

    pub fn add_startup_stage<H: Handle>(&mut self) {
        self.systems.add_startup_stage::<H>();
    }

    pub fn add_system_stage<H: Handle>(&mut self) {
        self.systems.add_systems_stage::<H>();
    }
}