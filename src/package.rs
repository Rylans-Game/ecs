
use super::handle::Component;
use super::anon::Anon;
use super::archetypes::Archetype;
use super::archetypes::ComponentId;
use super::archetypes::TableIndex;
use super::archetypes::Column;

pub struct Package {
    pub(crate) components: Vec<Anon>,
}

impl Package {
    pub fn new() -> Self {
        Self {
            components: Vec::with_capacity(8),
        }
    }

    pub(crate) fn archetype(&self) -> Archetype {
        let mut archetype = Archetype::new();
        for anon in self.components.iter() {
            archetype = archetype.add(anon.id() as u64);
        }
        archetype
    }

    pub(crate) fn remove(&mut self, id: ComponentId) {
        for i in 0..self.components.len() {
            if id == self.components[i].id() {
                self.components[i].clear();
                self.components.remove(i);
                return;
            }
        }
    }

    pub(crate) fn insert_anon(&mut self, anon: Anon) {
        for i in 0..self.components.len() {
            if anon.id() == self.components[i].id() {
                self.components[i].clear();
                self.components[i] = anon;
                return;
            }
        }

        self.components.push(anon);
    }

    pub(crate) fn pop(&mut self) -> Option<Anon> {
        self.components.pop()
    }

    pub fn with<C: Component>(mut self, cmp: C) -> Self {
        self.components.push(Anon::new::<C>(cmp));
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct PackageIndex {
    pub table: TableIndex,
    pub col: Column,
}

pub struct PackageIndexIter {
    pub table: TableIndex,
    pub col: usize,
    pub len: usize,
}

impl Iterator for PackageIndexIter {
    type Item = PackageIndex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.col == self.len {
            None
        } else {
            self.col += 1;
            Some(PackageIndex {
                table: self.table,
                col: self.col - 1,
            })
        }
    }
}

pub struct PackageIndexChain {
    pub iters: Vec<PackageIndexIter>,
}

impl PackageIndexChain {
    pub fn push(&mut self, iter: PackageIndexIter) {
        self.iters.push(iter)
    }

    pub fn len(&self) -> usize {
        let mut count = 0;
        for iter in self.iters.iter() {
            count += iter.len
        }
        count
    }
}

impl Iterator for PackageIndexChain {
    type Item = PackageIndex;

    fn next(&mut self) -> Option<Self::Item> {
        // get the last iter if it exists
        if let Some(curr) = self.iters.last_mut() {
            let out = curr.next();

            // if iters is empty now, move to the next one. 
            if curr.col == curr.len {
                self.iters.pop();
            }

            out   
        } else {
            // else, this iter is done. 
            None
        }
    }
}