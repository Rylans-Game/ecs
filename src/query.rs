
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use indexmap::IndexSet;

use crate::anon::Anon;

use super::ptr::Ptr;
use super::ecs::Ecs;
use super::error::Trace;
use super::scheduler::Accessor;
use super::handle::Component;
use super::anon::AnonIterChain;
use super::archetypes::TableIndex;
use super::package::PackageIndexChain;
use super::package::PackageIndex;
use super::archetypes::Archetype;
use super::handle::Handle;
use super::trace;

pub struct Query<Q: IntoQuery> {
    ecs: Ptr<Ecs>,
    marker: PhantomData<Q>,
}

pub trait IntoQuery: 'static {
    type Item: Iterator;

    fn into_query(ecs: Ptr<Ecs>) -> Trace<Self::Item>;
    fn accessors(a: &mut Vec<Accessor>, ecs: Ptr<Ecs>) -> Trace<()>;
}

pub trait QueryParam: 'static {
    type Item: Component;
    type Output;

    fn collect(indices: &IndexSet<TableIndex>, ecs: Ptr<Ecs>) -> AnonIterChain<Self::Item>;
    fn accessors(accessors: &mut Vec<Accessor>);
    fn wrap(data: &'static mut Self::Item) -> Self::Output;
}

pub struct Ref<C: Component>(&'static C);

impl<C: Component> Deref for Ref<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<C: Component> QueryParam for Ref<C> {
    type Item = C;

    type Output = Ref<C>;

    fn collect(indices: &IndexSet<TableIndex>, ecs: Ptr<Ecs>) -> AnonIterChain<Self::Item> {
        ecs.archetypes.collect(indices)
    }

    fn accessors(accessors: &mut Vec<Accessor>) {
        if *C::handle() == u16::MAX {
            panic!("Tried to collect accessors for undeclared Component {} in a Query!", std::any::type_name::<C>());
        }

        accessors.push(Accessor::Ref(*C::handle()))
    }

    fn wrap(data: &'static mut Self::Item) -> Self::Output {
        Self(data)
    }
}

pub struct Mut<C: Component>(&'static mut C);

impl<C: Component> Deref for Mut<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<C: Component> DerefMut for Mut<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<C: Component> QueryParam for Mut<C> {
    type Item = C;

    type Output = Mut<C>;

    fn collect(indices: &IndexSet<TableIndex>, ecs: Ptr<Ecs>) -> AnonIterChain<Self::Item> {
        ecs.archetypes.collect(indices)
    }

    fn accessors(accessors: &mut Vec<Accessor>) {
        if *C::handle() == u16::MAX {
            panic!("Tried to collect accessors for undeclared Component {} in a Query!", std::any::type_name::<C>());
        }

        accessors.push(Accessor::Mut(*C::handle()))
    }

    fn wrap(data: &'static mut Self::Item) -> Self::Output {
        Self(data)
    }
}

pub struct Query1<T1> 
where
    T1: QueryParam,
{
    t1: AnonIterChain<T1>,
    p: PackageIndexChain,
}

impl<T1> Iterator for Query1<T1>
where
    T1: QueryParam<Item=T1>,
{
    type Item = (T1::Output, PackageIndex);

    fn next(&mut self) -> Option<Self::Item> {
        if self.t1.iters.len() > 0 {
            Some((
                T1::wrap(self.t1.next().unwrap()),
                self.p.next().unwrap(),
            ))
        } else {
            None
        }
    }
}

impl<T1> IntoQuery for (T1,) 
where
    T1: QueryParam<Item=T1>,
{
    type Item = Query1<T1>;

    fn into_query(ecs: Ptr<Ecs>) -> Trace<Self::Item> {
        let mut arch = Archetype::new();
        arch = arch.add(*T1::Item::handle() as u64);

        let indices = trace!(ecs.archetypes.query_cache(arch));

        Trace::Ok(Query1 {
            t1: T1::collect(indices, ecs.clone()),
            p: ecs.archetypes.collect_indices(indices),
        })
    }

    fn accessors(a: &mut Vec<Accessor>, ecs: Ptr<Ecs>) -> Trace<()> {
        let mut arch = Archetype::new();
        arch = arch.add(*T1::Item::handle() as u64);

        let mut ids = Vec::new();
        ids.push(*T1::Item::handle());

        T1::accessors(a);

        unsafe { (*ecs.get_mut()).archetypes.load_query(arch, ids) }
    }
}

pub struct Query2<T1, T2> 
where
    T1: QueryParam,
    T2: QueryParam,
{
    t1: AnonIterChain<T1>,
    t2: AnonIterChain<T2>,
    p: PackageIndexChain,
}

impl<T1, T2> Iterator for Query2<T1, T2>
where
    T1: QueryParam<Item=T1>,
    T2: QueryParam<Item=T2>,
{
    type Item = (T1::Output, T2::Output, PackageIndex);

    fn next(&mut self) -> Option<Self::Item> {
        if self.t1.iters.len() > 0 {
            Some((
                T1::wrap(self.t1.next().unwrap()),
                T2::wrap(self.t2.next().unwrap()),
                self.p.next().unwrap(),
            ))
        } else {
            None
        }
    }
}

impl<T1, T2> IntoQuery for (T1, T2) 
where
    T1: QueryParam<Item=T1>,
    T2: QueryParam<Item=T2>,
{
    type Item = Query2<T1, T2>;

    fn into_query(ecs: Ptr<Ecs>) -> Trace<Self::Item> {
        let mut arch = Archetype::new();
        arch = arch.add(*T1::Item::handle() as u64);
        arch = arch.add(*T2::Item::handle() as u64);

        let indices = trace!(ecs.archetypes.query_cache(arch));

        Trace::Ok(Query2 {
            t1: T1::collect(indices, ecs.clone()),
            t2: T2::collect(indices, ecs.clone()),
            p: ecs.archetypes.collect_indices(indices),
        })
    }

    fn accessors(a: &mut Vec<Accessor>, ecs: Ptr<Ecs>) -> Trace<()> {
        let mut arch = Archetype::new();
        arch = arch.add(*T1::Item::handle() as u64);
        arch = arch.add(*T2::Item::handle() as u64);

        let mut ids = Vec::new();
        ids.push(*T1::Item::handle());
        ids.push(*T2::Item::handle());

        T1::accessors(a);
        T2::accessors(a);

        unsafe { (*ecs.get_mut()).archetypes.load_query(arch, ids) }
    }
}

pub struct Query3<T1, T2, T3> 
where
    T1: QueryParam,
    T2: QueryParam,
    T3: QueryParam,
{
    t1: AnonIterChain<T1>,
    t2: AnonIterChain<T2>,
    t3: AnonIterChain<T3>,
    p: PackageIndexChain,
}

impl<T1, T2, T3> Iterator for Query3<T1, T2, T3>
where
    T1: QueryParam<Item=T1>,
    T2: QueryParam<Item=T2>,
    T3: QueryParam<Item=T3>,
{
    type Item = (T1::Output, T2::Output, T3::Output, PackageIndex);

    fn next(&mut self) -> Option<Self::Item> {
        if self.t1.iters.len() > 0 {
            Some((
                T1::wrap(self.t1.next().unwrap()),
                T2::wrap(self.t2.next().unwrap()),
                T3::wrap(self.t3.next().unwrap()),
                self.p.next().unwrap(),
            ))
        } else {
            None
        }
    }
}

impl<T1, T2, T3> IntoQuery for (T1, T2, T3) 
where
    T1: QueryParam<Item=T1>,
    T2: QueryParam<Item=T2>,
    T3: QueryParam<Item=T3>,
{
    type Item = Query3<T1, T2, T3>;

    fn into_query(ecs: Ptr<Ecs>) -> Trace<Self::Item> {
        let mut arch = Archetype::new();
        arch = arch.add(*T1::Item::handle() as u64);
        arch = arch.add(*T2::Item::handle() as u64);
        arch = arch.add(*T3::Item::handle() as u64);

        let indices = trace!(ecs.archetypes.query_cache(arch));

        Trace::Ok(Query3 {
            t1: T1::collect(indices, ecs.clone()),
            t2: T2::collect(indices, ecs.clone()),
            t3: T3::collect(indices, ecs.clone()),
            p: ecs.archetypes.collect_indices(indices),
        })
    }

    fn accessors(a: &mut Vec<Accessor>, ecs: Ptr<Ecs>) -> Trace<()> {
        let mut arch = Archetype::new();
        arch = arch.add(*T1::Item::handle() as u64);
        arch = arch.add(*T2::Item::handle() as u64);
        arch = arch.add(*T3::Item::handle() as u64);

        let mut ids = Vec::new();
        ids.push(*T1::Item::handle());
        ids.push(*T2::Item::handle());
        ids.push(*T3::Item::handle());

        T1::accessors(a);
        T2::accessors(a);
        T3::accessors(a);

        unsafe { (*ecs.get_mut()).archetypes.load_query(arch, ids) }
    }
}

pub struct Query4<T1, T2, T3, T4> 
where
    T1: QueryParam,
    T2: QueryParam,
    T3: QueryParam,
    T4: QueryParam,
{
    t1: AnonIterChain<T1>,
    t2: AnonIterChain<T2>,
    t3: AnonIterChain<T3>,
    t4: AnonIterChain<T4>,
    p: PackageIndexChain,
}

impl<T1, T2, T3, T4> Iterator for Query4<T1, T2, T3, T4>
where
    T1: QueryParam<Item=T1>,
    T2: QueryParam<Item=T2>,
    T3: QueryParam<Item=T3>,
    T4: QueryParam<Item=T4>,
{
    type Item = (T1::Output, T2::Output, T3::Output, T4::Output, PackageIndex);

    fn next(&mut self) -> Option<Self::Item> {
        if self.t1.iters.len() > 0 {
            Some((
                T1::wrap(self.t1.next().unwrap()),
                T2::wrap(self.t2.next().unwrap()),
                T3::wrap(self.t3.next().unwrap()),
                T4::wrap(self.t4.next().unwrap()),
                self.p.next().unwrap(),
            ))
        } else {
            None
        }
    }
}

impl<T1, T2, T3, T4> IntoQuery for (T1, T2, T3, T4) 
where
    T1: QueryParam<Item=T1>,
    T2: QueryParam<Item=T2>,
    T3: QueryParam<Item=T3>,
    T4: QueryParam<Item=T4>,
{
    type Item = Query4<T1, T2, T3, T4>;

    fn into_query(ecs: Ptr<Ecs>) -> Trace<Self::Item> {
        let mut arch = Archetype::new();
        arch = arch.add(*T1::Item::handle() as u64);
        arch = arch.add(*T2::Item::handle() as u64);
        arch = arch.add(*T3::Item::handle() as u64);
        arch = arch.add(*T4::Item::handle() as u64);

        let indices = trace!(ecs.archetypes.query_cache(arch));

        Trace::Ok(Query4 {
            t1: T1::collect(indices, ecs.clone()),
            t2: T2::collect(indices, ecs.clone()),
            t3: T3::collect(indices, ecs.clone()),
            t4: T4::collect(indices, ecs.clone()),
            p: ecs.archetypes.collect_indices(indices),
        })
    }

    fn accessors(a: &mut Vec<Accessor>, ecs: Ptr<Ecs>) -> Trace<()> {
        let mut arch = Archetype::new();
        arch = arch.add(*T1::Item::handle() as u64);
        arch = arch.add(*T2::Item::handle() as u64);
        arch = arch.add(*T3::Item::handle() as u64);
        arch = arch.add(*T4::Item::handle() as u64);

        let mut ids = Vec::new();
        ids.push(*T1::Item::handle());
        ids.push(*T2::Item::handle());
        ids.push(*T3::Item::handle());
        ids.push(*T4::Item::handle());

        T1::accessors(a);
        T2::accessors(a);
        T3::accessors(a);
        T4::accessors(a);

        unsafe { (*ecs.get_mut()).archetypes.load_query(arch, ids) }
    }
}