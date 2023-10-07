
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::any::type_name;

use super::handle::{Resource, Component};
use super::error::Trace;
use super::scheduler::Accessor;
use super::ecs::Ecs;
use super::ptr::Ptr;
use crate::{start_trace, trace};

pub trait System: Default {
    fn execute(self) -> Trace<()>;
}

pub trait Fetch: Default {
    fn fetch(ecs: Ptr<Ecs>) -> Trace<Self>;
    fn access(v: &mut Vec<Accessor>) -> Trace<()>;
}

pub struct ResRef<R>(pub(crate) *const R);

impl<R: Resource> Deref for ResRef<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.0) }
    }
}

impl<R: Resource> Default for ResRef<R> {
    fn default() -> Self {
        Self(std::ptr::null())
    }
}

impl<R: Resource> Fetch for ResRef<R> {
    fn fetch(ecs: Ptr<Ecs>) -> Trace<Self> {
        ecs.get_resource_ref::<R>()
    }

    fn access(v: &mut Vec<Accessor>) -> Trace<()> {
        if *R::handle() == u16::MAX {
            start_trace!(format!("Tried to collect accessor for resource {}, but it had not been declared!", type_name::<R>()))
        }

        v.push(Accessor::ResRef(*R::handle()));

        Trace::Ok(())
    }
}

pub struct ResMut<R: Resource>(pub(crate) *mut R);

impl<R: Resource> Deref for ResMut<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.0) }
    }
}

impl<R: Resource> DerefMut for ResMut<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.0) }
    }
}

impl<R: Resource> Default for ResMut<R> {
    fn default() -> Self {
        Self(std::ptr::null_mut())
    }
}

impl<R: Resource> Fetch for ResMut<R> {
    fn fetch(ecs: Ptr<Ecs>) -> Trace<Self> {
        ecs.get_resource_mut::<R>()
    }

    fn access(v: &mut Vec<Accessor>) -> Trace<()> {
        if *R::handle() == u16::MAX {
            start_trace!(format!("Tried to collect accessor for resource {}, but it had not been declared!", type_name::<R>()))
        }

        v.push(Accessor::ResMut(*R::handle()));

        Trace::Ok(())
    }
}

impl<P1> Fetch for (P1,) 
where
    P1: Fetch,
{
    fn fetch(ecs: Ptr<Ecs>) -> Trace<Self> {
        Trace::Ok((trace!(P1::fetch(ecs)),))
    }

    fn access(v: &mut Vec<Accessor>) -> Trace<()> {
        P1::access(v)
    }
}

impl<P1, P2> Fetch for (P1, P2) 
where
    P1: Fetch,
    P2: Fetch,
{
    fn fetch(ecs: Ptr<Ecs>) -> Trace<Self> {
        Trace::Ok((
            trace!(P1::fetch(ecs.clone())), 
            trace!(P2::fetch(ecs.clone()))
        ))
    }

    fn access(v: &mut Vec<Accessor>) -> Trace<()> {
        trace!(P1::access(v));
        trace!(P2::access(v));

        Trace::Ok(())
    }
}

impl<P1, P2, P3> Fetch for (P1, P2, P3) 
where
    P1: Fetch,
    P2: Fetch,
    P3: Fetch,
{
    fn fetch(ecs: Ptr<Ecs>) -> Trace<Self> {
        Trace::Ok((
            trace!(P1::fetch(ecs.clone())), 
            trace!(P2::fetch(ecs.clone())),
            trace!(P3::fetch(ecs.clone())),
        ))
    }

    fn access(v: &mut Vec<Accessor>) -> Trace<()> {
        trace!(P1::access(v));
        trace!(P2::access(v));
        trace!(P3::access(v));

        Trace::Ok(())
    }
}

impl<P1, P2, P3, P4> Fetch for (P1, P2, P3, P4)
where
    P1: Fetch,
    P2: Fetch,
    P3: Fetch,
    P4: Fetch,
{
    fn fetch(ecs: Ptr<Ecs>) -> Trace<Self> {
        Trace::Ok((
            trace!(P1::fetch(ecs.clone())), 
            trace!(P2::fetch(ecs.clone())),
            trace!(P3::fetch(ecs.clone())),
            trace!(P4::fetch(ecs.clone())),
        ))
    }

    fn access(v: &mut Vec<Accessor>) -> Trace<()> {
        trace!(P1::access(v));
        trace!(P2::access(v));
        trace!(P3::access(v));
        trace!(P4::access(v));

        Trace::Ok(())
    }
}

impl<P1, P2, P3, P4, P5> Fetch for (P1, P2, P3, P4, P5)
where
    P1: Fetch,
    P2: Fetch,
    P3: Fetch,
    P4: Fetch,
    P5: Fetch,
{
    fn fetch(ecs: Ptr<Ecs>) -> Trace<Self> {
        Trace::Ok((
            trace!(P1::fetch(ecs.clone())), 
            trace!(P2::fetch(ecs.clone())),
            trace!(P3::fetch(ecs.clone())),
            trace!(P4::fetch(ecs.clone())),
            trace!(P5::fetch(ecs.clone())),
        ))
    }

    fn access(v: &mut Vec<Accessor>) -> Trace<()> {
        trace!(P1::access(v));
        trace!(P2::access(v));
        trace!(P3::access(v));
        trace!(P4::access(v));
        trace!(P5::access(v));

        Trace::Ok(())
    }
}

impl<P1, P2, P3, P4, P5, P6> Fetch for (P1, P2, P3, P4, P5, P6)
where
    P1: Fetch,
    P2: Fetch,
    P3: Fetch,
    P4: Fetch,
    P5: Fetch,
    P6: Fetch,
{
    fn fetch(ecs: Ptr<Ecs>) -> Trace<Self> {
        Trace::Ok((
            trace!(P1::fetch(ecs.clone())), 
            trace!(P2::fetch(ecs.clone())),
            trace!(P3::fetch(ecs.clone())),
            trace!(P4::fetch(ecs.clone())),
            trace!(P5::fetch(ecs.clone())),
            trace!(P6::fetch(ecs.clone())),
        ))
    }

    fn access(v: &mut Vec<Accessor>) -> Trace<()> {
        trace!(P1::access(v));
        trace!(P2::access(v));
        trace!(P3::access(v));
        trace!(P4::access(v));
        trace!(P5::access(v));
        trace!(P6::access(v));

        Trace::Ok(())
    }
}