
use std::any::type_name;

use super::error::Trace;
use super::params::Fetch;
use super::scheduler::Scheduler;
use super::handle::Handle;
use super::params::System;

pub struct Systems {
    startup: Vec<Scheduler>,   
    systems: Vec<Scheduler>,
}

impl Systems {
    pub fn new() -> Self {
        Self {
            startup: Vec::new(),
            systems: Vec::new(),
        }
    }

    pub fn add_startup_stage<H: Handle>(&mut self) {

        if *H::handle() == u16::MAX {
            panic!("Tried to add the same startup stage handle twice! Name: {}", type_name::<H>());
        }

        *H::handle() = self.startup.len() as u16;
        self.startup.push(Scheduler::new(type_name::<H>()));
    }

    pub fn add_systems_stage<H: Handle>(&mut self) {

        if *H::handle() == u16::MAX {
            panic!("Tried to add the same system stage handle twice! Name: {}", type_name::<H>());
        }

        *H::handle() = self.systems.len() as u16;
        self.systems.push(Scheduler::new(type_name::<H>()));
    }

    pub fn add_startup<S: System + Fetch, H: Handle>(&mut self) {
        if *H::handle() == u16::MAX {
            panic!("Tried to add startup system {} to stage {}, which has not been declared!"
            , type_name::<S>(), type_name::<H>());
        }

        if *H::handle() >= self.startup.len() as u16 {
            panic!("Tried to add startup system {} to stage {}, which is invalid! 
            (handle contained a value greater than the stage limit)", type_name::<S>(), type_name::<H>());
        }

        self.startup[*H::handle() as usize].add_system::<S>();
    }

    pub fn add_system<S: System + Fetch, H: Handle>(&mut self) {
        if *H::handle() == u16::MAX {
            panic!("Tried to add system {} to stage {}, which has not been declared!"
            , type_name::<S>(), type_name::<H>());
        }

        if *H::handle() >= self.startup.len() as u16 {
            panic!("Tried to add system {} to stage {}, which is invalid! 
            (handle contained a value greater than the stage limit)", type_name::<S>(), type_name::<H>());
        }

        self.systems[*H::handle() as usize].add_system::<S>();
    }

    pub fn execute_startup(&self) {

    }
}





