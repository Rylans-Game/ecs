
use rayon::prelude::*;
use indexmap::IndexMap;
use super::handle::Component;
use super::archetypes::ComponentId;
use super::anon::AnonVec;
use super::package::Package;
use super::archetypes::Column;
use super::anon::AnonIter;
use super::package::PackageIndexIter;
use super::commands::Modify;

pub struct Table {
    rows: IndexMap<ComponentId, AnonVec>,
    len: usize,
}

impl Table {
    pub fn new(mut packages: Vec<Package>) -> Self {
        let len = packages.len();
        let mut rows = IndexMap::new();

        if let Some(mut package) = packages.pop() {
            while let Some(anon) = package.pop() {
                rows.insert(anon.id(), AnonVec::new(anon));
            }
        } else {
            panic!("Attempted to spawn an empty package vector!")
        }

        while let Some(mut package) = packages.pop() {
            while let Some(anon) = package.pop() {
                if let Some(row) = rows.get_mut(&anon.id()) {
                    row.push(anon);
                } else {
                    panic!("Invalid Package! ID Not Found!")
                }
            }
        }

        Self {
            rows, len, 
        }
    }

    pub fn spawn(&mut self, mut packages: Vec<Package>) {
        self.len += packages.len();

        while let Some(mut package) = packages.pop() {
            while let Some(anon) = package.pop() {
                if let Some(row) = self.rows.get_mut(&anon.id()) {
                    row.push(anon);
                } else {
                    panic!("Attempted to spawn invalid component!")
                }
            }
        }
    }

    pub fn destroy(&mut self, mut destroys: Vec<(Column, bool)>) {
        self.len -= destroys.len();

        destroys.par_sort_by(|a, b| {
            a.0.partial_cmp(&b.0).unwrap()
        });

        destroys.dedup();

        while let Some((col, drop)) = destroys.pop() {
            if drop == NO_DROP {
                for (_, row) in self.rows.iter_mut() {
                    row.destroy_nodrop(col);
                }
            } else {
                for (_, row) in self.rows.iter_mut() {
                    row.destroy_swap(col);
                }
            }
        }
    }

    pub fn contains(&self, ids: &Vec<ComponentId>) -> bool {
        for id in ids.iter() {
            if !self.rows.contains_key(id) { return false }
        }
        true
    }

    pub fn collect<C: Component>(&self) -> Option<AnonIter<C>> {
        if self.len == 0 { return None }

        if let Some(row) = self.rows.get(C::handle()) {
            return Some(row.iter_as::<C>())
        } else {
            panic!("Attempted to collect component from archetype in which it does not exist")
        }
    }

    pub fn collect_indices(&self, table: usize) -> Option<PackageIndexIter> {
        if self.len == 0 {
            None
        } else {
            Some(PackageIndexIter {
                table,
                col: 0,
                len: self.len,
            })
        }
    }

    pub fn extract(&self, mut modify: Modify, col: Column) -> Package {
        let mut package = Package::new();
        for (_, row) in self.rows.iter() {
            package.insert_anon(row.index(col));
        }

        while let Some(destroy) = modify.remove.pop() {
            package.remove(destroy);
        }

        while let Some(insert) = modify.insert.pop() {
            package.insert_anon(insert);
        }

        package
    }
}

unsafe impl Send for Table {}
unsafe impl Sync for Table {}

pub const NO_DROP: bool = false;
pub const DO_DROP: bool = true;