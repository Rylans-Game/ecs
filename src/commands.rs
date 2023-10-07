
use std::collections::BTreeMap;

use super::archetypes::Archetype;
use super::package::Package;
use super::archetypes::TableIndex;
use super::archetypes::Column;
use super::package::PackageIndex;
use super::table::{DO_DROP, NO_DROP};
use super::archetypes::ComponentId;
use super::anon::Anon;
use super::handle::Component;
use super::ptr::Ptr;
use super::ecs::Ecs;
use super::params::Fetch;
use super::error::Trace;
use super::scheduler::Accessor;
use super::trace;

pub struct Commands {
    pub(crate) ecs: Ptr<Ecs>,
    pub(crate) spawn: BTreeMap<Archetype, Vec<Package>>,
    pub(crate) destroy: BTreeMap<TableIndex, Vec<(Column, bool)>>,
    pub(crate) modify: BTreeMap<PackageIndex, Modify>,
}

impl Commands {
    pub(crate) fn new(ecs: Ptr<Ecs>) -> Self {
        Self {
            ecs,
            spawn: BTreeMap::new(),
            destroy: BTreeMap::new(),
            modify: BTreeMap::new(),
        }
    }

    pub(crate) const fn null() -> Self {
        Self {
            ecs: Ptr::null(),
            spawn: BTreeMap::new(),
            destroy: BTreeMap::new(),
            modify: BTreeMap::new(),
        }
    }

    pub(crate) fn spawn_package(&mut self, package: Package) {
        let key = package.archetype();

        if let Some(packages) = self.spawn.get_mut(&key) {
            packages.push(package);
        } else {
            if let Some(_) = self.spawn.insert(key, vec![package]) {
                panic!("Attempted to insert the same key twice!")
            }
        }
    }

    pub(crate) fn destroy_nodrop(&mut self, index: PackageIndex) {
        if let Some(des) = self.destroy.get_mut(&index.table) {
            des.push((index.col, DO_DROP));
        } else {
            if let Some(_) = self.destroy.insert(index.table, vec![(index.col, DO_DROP)]) {
                panic!("Attempted to insert the same key twice! (how did you even do this?)");
            }
        }
    }

    pub fn spawn<F>(&mut self, predicate: F) 
    where   
        F: Fn() -> Package
    {
        self.spawn_package(predicate());
    }

    pub fn destroy(&mut self, index: PackageIndex) {
        if let Some(des) = self.destroy.get_mut(&index.table) {
            des.push((index.col, DO_DROP));
        } else {
            if let Some(_) = self.destroy.insert(index.table, vec![(index.col, DO_DROP)]) {
                panic!("Attempted to insert the same key twice! (how did you even do this?)");
            }
        }
    }

    pub fn modify<F>(&mut self, index: PackageIndex, predicate: F)
    where
        F: Fn(&mut Modify)
    {
        if let Some(modify) = self.modify.get_mut(&index) {
            predicate(modify)
        } else {
            let mut modify = Modify::new();
            predicate(&mut modify);
            if let Some(_) = self.modify.insert(index, modify) {
                panic!("Attempted to insert the same key twice!")
            }
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.spawn.is_empty() &&
        self.destroy.is_empty() &&
        self.modify.is_empty()
    }

    pub fn submit(self) -> Trace<()> {
        if !self.is_empty() {
            let ecs = self.ecs.get_mut();
                unsafe {trace!((*ecs).archetypes.submit_commands(self)); }
        }

        Trace::Ok(())
    }
}

impl Default for Commands {
    fn default() -> Self {
        Self { 
            ecs: Ptr::null(),
            spawn: BTreeMap::new(),
            destroy: BTreeMap::new(),
            modify: BTreeMap::new(),
        }
    }
}

pub struct Modify {
    pub(crate) insert: Vec<Anon>,
    pub(crate) remove: Vec<ComponentId>,
}

impl Modify {
    pub(crate) fn new() -> Self {
        Self {
            insert: Vec::with_capacity(8),
            remove: Vec::with_capacity(8),
        }
    }

    pub(crate) fn extend(&mut self, modify: Modify) {
        self.insert.extend(modify.insert);
        self.remove.extend(modify.remove);
    }

    pub fn with<C: Component>(&mut self, cmp: C) -> &mut Self {
        self.insert.push(Anon::new::<C>(cmp));
        self
    }

    pub fn without<C: Component>(&mut self) -> &mut Self {
        self.remove.push(*C::handle());
        self
    }
}

impl Fetch for Commands {
    fn fetch(ecs: Ptr<Ecs>) -> Trace<Self> {
        Trace::Ok(Commands::new(ecs))
    }

    fn access(_: &mut Vec<Accessor>) -> Trace<()> {
        Trace::Ok(())
        // we dont access anything un-safely. 
    }
}