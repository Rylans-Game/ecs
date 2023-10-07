
use std::collections::BTreeMap;
use std::sync::Mutex;

use rayon::prelude::*;
use indexmap::IndexSet;
use super::handle::Component;
use anyhow::{Result, anyhow};
use super::commands::Commands;
use super::table::Table;
use super::table::{DO_DROP, NO_DROP};
use super::anon::AnonIterChain;
use super::package::PackageIndexChain;
use super::error::Trace;
use super::start_trace;

pub type Column = usize;
pub type TableIndex = usize;
pub type ComponentId = u16;

pub struct Archetypes {
    archetypes: BTreeMap<Archetype, TableIndex>,
    commands: Commands,
    tables: Vec<Table>,
    cache: QueryCache,
}

impl Archetypes {
    pub const fn new() -> Self {
        Self {
            /// Fast lookup for inserting Archetypes into Tables. 
            archetypes: BTreeMap::new(),

            /// Command Queue. Commands can be submitted and stored
            /// here before being processed. 
            commands: Commands::null(),

            /// Stores the tables, which stores the components
            /// for a specific archetype. 
            tables: Vec::new(),

            /// Cache for storing archetypes that are parents
            /// of Queries. Updated whenever a new table is allocated. 
            cache: QueryCache::new(),
        }
    }

    pub fn load_query(&mut self, key: Archetype, ids: Vec<ComponentId>) -> Trace<()> {
        self.cache.load(key, ids)
    }

    pub fn query_cache(&self, key: Archetype) -> Trace<&IndexSet<TableIndex>> {
        self.cache.search(key)
    }

    pub fn submit_commands(&mut self, mut commands: Commands) -> Trace<()> {
        while let Some((key, packages)) = commands.spawn.pop_first() {
            if let Some(spawn) = self.commands.spawn.get_mut(&key) {
                spawn.extend(packages);
            } else {
                if let Some(_) = self.commands.spawn.insert(key, packages) {
                    start_trace!("Attempted to insert the same key into spawn twice!, (internal error, contact maintainer)");
                }
            }
        }

        while let Some((table, columns)) = commands.destroy.pop_first() {
            if let Some(destroy) = self.commands.destroy.get_mut(&table) {
                destroy.extend(columns);
            } else {
                if let Some(_) = self.commands.destroy.insert(table, columns) {
                    start_trace!("Tried to insert the same destroy request twice! (internal error, contact maintainer)")
                }
            }
        }

        while let Some((index, modifies)) = commands.modify.pop_first() {
            if let Some(modify) = self.commands.modify.get_mut(&index) {
                modify.extend(modifies);
            } else {
                if let Some(_) = self.commands.modify.insert(index, modifies) {
                    start_trace!("Tried modify the same Modify request twice (internal error, contact maintaner)")
                }
            }
        }

        Trace::Ok(())
    }

    pub fn flush_queues(&mut self) {
        while let Some((index, modify)) = self.commands.modify.pop_first() {
            if let Some(columns) = self.commands.destroy.get_mut(&index.table) {
                if !columns.contains(&(index.col, DO_DROP)) {
                    let package = self.tables[index.table].extract(modify, index.col);
                    self.commands.spawn_package(package);
                    self.commands.destroy_nodrop(index);
                } 
            }
        }

        while let Some((key, packages)) = self.commands.spawn.pop_first() {
            if let Some(index) = self.archetypes.get(&key) {
                self.tables[*index].spawn(packages);
            } else {
                let len = self.tables.len();
                if let None = self.archetypes.insert(key, len) {
                    self.tables.push(Table::new(packages));
                    self.cache.update(len, &self.tables[len]);
                } else {
                    panic!("Attempted to insert a duplicate archetype!")
                }
            }
        }

        while let Some((index, destroys)) = self.commands.destroy.pop_first() {
            self.tables[index].destroy(destroys);
        }
    }

    pub fn collect<C: Component>(&self, indices: &IndexSet<TableIndex>) -> AnonIterChain<C> {
        let mut chain = AnonIterChain { iters: Vec::with_capacity(indices.len()) };

        for index in indices.iter() {
            if let Some(iter) = self.tables[*index].collect::<C>() {
                chain.push(iter);
            }
        }

        chain
    }

    pub fn collect_indices(&self, indices: &IndexSet<TableIndex>) -> PackageIndexChain {
        let mut chain = PackageIndexChain { iters: Vec::with_capacity(indices.len()) };

        for index in indices.iter() {
            if let Some(iter) = self.tables[*index].collect_indices(*index) {
                chain.push(iter);
            }
        }

        chain
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Archetype(u64);

impl Archetype {
    pub const fn new() -> Self {
        Self(1)
    }

    pub const fn add(mut self, id: u64) -> Self {
        let h = id.wrapping_mul(123456789123456789);
        self.0 = self.0.wrapping_add(id.wrapping_add(h.wrapping_shl((h % 32) as u32 + 1)));
        self
    }
}

pub struct QueryCache {
    cache: BTreeMap<Archetype, (Vec<ComponentId>, IndexSet<TableIndex>)>,
}

impl QueryCache {
    pub const fn new() -> Self {
        Self {
            cache: BTreeMap::new(),
        }
    }

    pub fn update(&mut self, index: TableIndex, table: &Table) {
        self.cache.par_iter_mut().for_each(|(_, (ids, indices))| {
            if table.contains(ids) {
                indices.insert(index);
            }
        });
    }

    pub fn search(&self, key: Archetype) -> Trace<&IndexSet<TableIndex>> {
        if let Some((_, indices)) = self.cache.get(&key) {
            Trace::Ok(indices)
        } else {
            start_trace!(format!("Tried to search the Query Cache for a Query that does not exist!"))
        }
    }

    pub fn load(&mut self, key: Archetype, ids: Vec<ComponentId>) -> Trace<()> {
        if let Some((exists, _)) = self.cache.get(&key) {
            for id in ids.iter() {
                if !exists.contains(id) {
                    start_trace!("
                        Same Archetype but different components! 
                        Indicates hashing algorithm is bad. 
                        (internal error, contact maintaner)
                    ");
                }
            }
        } else {
            self.cache.insert(key, (ids, IndexSet::new()));
        }

        Trace::Ok(())
    }
}