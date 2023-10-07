
use std::any::Any;

use super::handle::Resource;
use super::ptr::Unsafe;
use super::error::Trace;
use crate::start_trace;
use super::params::{ResMut, ResRef};

pub struct Resources {
    resources: Vec<Box<dyn Any>>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
        }
    }

    pub fn add_resource<R: Resource>(&mut self, resource: R) {
        *R::handle() = self.resources.len() as u16;
        self.resources.push(Box::new(Unsafe::new(resource)));
    }

    pub fn get_resource_ref<R: Resource>(&self) -> Trace<ResRef<R>> {
        if *R::handle() >= self.resources.len() as u16 {
            start_trace!(format!("Handle for resource {} has not been initialized! Did you forget to declare it?", R::name()));
        }

        if let Some(res) = self.resources[*R::handle() as usize].downcast_ref::<Unsafe<R>>() {
            Trace::Ok(ResRef(res.get()))
        } else {
            start_trace!(format!("Handle for resource {} was invalid!", R::name()));
        }
    }

    pub fn get_resource_mut<R: Resource>(&self) -> Trace<ResMut<R>> {
        if *R::handle() >= self.resources.len() as u16 {
            start_trace!(format!("Handle for resource {} has not been initialized! Did you forget to declare it?", R::name()));
        }

        if let Some(res) = self.resources[*R::handle() as usize].downcast_ref::<Unsafe<R>>() {
            Trace::Ok(ResMut(res.get()))
        } else {
            start_trace!(format!("Handle for resource {} was invalid!", R::name()));
        }
    }
}