
use std::any::type_name;

use rayon::prelude::*;

use super::error::Trace;
use super::params::System;
use super::ptr::Ptr;
use super::ecs::Ecs;
use super::params::Fetch;
use super::trace;

pub struct Scheduler {
    stage: &'static str,
    temp: Vec<Node>,
    groups: Vec<Group>,
}

impl Scheduler {
    pub fn new(stage: &'static str) -> Self {
        Self {
            stage,
            temp: Vec::new(),
            groups: Vec::new(),
        }
    }

    pub fn add_system<S: System + Fetch>(&mut self) {
        self.temp.push(
            Node {
                execute: |ecs| { S::execute(trace!(S::fetch(ecs))) },
                access: |accessors, ecs| { S::access(accessors, ecs) },
                accessors: Vec::new(),
                edges: Vec::new(),
                name: type_name::<S>(),
            }
        );
    }

    pub fn finalize(&mut self, ecs: Ptr<Ecs>) {
        // Compute Queries & Accessors for Each System
        for node in self.temp.iter_mut() {
            if let Trace::Err(e) = (node.access)(&mut node.accessors, ecs.clone()) {
                panic!("System Scheduler for stage {} encountered error in system {} with trace: \n {}", self.stage, node.name, e);
            }
        }

        // Compute all compatible systems.
        // for every node...
        for i in 0..self.temp.len() {
            // ...for every node after i...
            for other in (i+1)..self.temp.len() {
                // ...if the nodes do not conflict...
                if !conflicts(&self.temp[i], &self.temp[other]) {
                    // ...add other as an edge.
                    self.temp[i].edges.push(other);
                }
            }
        }

        // Sort the systems in order from most to least compatability
        self.temp.sort_by(|a, b| {
            b.edges.len().partial_cmp(&a.edges.len()).unwrap()
        });

        // for every node... (in reverse order of compatability)
        'l1: while let Some(node) = self.temp.pop() {
            // for every group...
            'l2: for group in self.groups.iter_mut() {
                // ...check if the node is compatable.
                for system in group.systems.iter() {
                    // if its not, go to the next group.
                    if conflicts(system, &node) {
                        continue 'l2;
                    }
                }

                // if the node is compatible
                // push the node to the group.
                group.systems.push(node);
                continue 'l1;
            }

            // if no compatible group is found, push a new group.
            self.groups.push(Group {
                systems: vec![node]
            })
        }
    }

    pub fn execute(&self, ecs: Ptr<Ecs>) {
        // Execute the groups
        for group in self.groups.iter() {
            match group.systems.len() {
                1 => {
                    // Execute on main thread, foregoing
                    // any overhead from launching threads.
                    if let Trace::Err(e) = (group.systems[0].execute)(ecs.clone()) {
                        panic!("System Scheduler for stage {} encountered error in system {} with trace: \n {}"
                        , self.stage,group.systems[0].name, e);
                    }
                },
                2 => {
                    // use join if there are only 2 systems
                    rayon::join(
                        || if let Trace::Err(e) = (group.systems[0].execute)(ecs.clone()) {
                            panic!("System Scheduler for stage {} encountered error in system {} with trace: \n {}"
                            , self.stage, group.systems[0].name, e);
                        },
                        || if let Trace::Err(e) = (group.systems[1].execute)(ecs.clone()) {
                            panic!("System Scheduler for stage {} encountered error in system {} with trace: \n {}"
                            , self.stage, group.systems[1].name, e);
                        }
                    );
                },
                _ => {
                    // use par_iter for any length larger than 2.
                    group.systems.par_iter().for_each(|node| {
                        if let Trace::Err(e) = (node.execute)(ecs.clone()) {
                            panic!("System Scheduler for stage {} encountered error in system {} with trace: \n {}"
                            , self.stage, node.name, e);
                        }
                    });
                }
            }
        }
    }
}

struct Group {
    systems: Vec<Node>,
}

struct Node {
    execute: fn(Ptr<Ecs>) -> Trace<()>,
    access: fn(&mut Vec<Accessor>, ecs: Ptr<Ecs>) -> Trace<()> ,
    accessors: Vec<Accessor>,
    edges: Vec<usize>,
    name: &'static str,
}

#[derive(PartialEq)]
pub enum Accessor {
    ResMut(u16),
    ResRef(u16),
    Ref(u16),
    Mut(u16),
}

fn conflicts(nodea: &Node, nodeb: &Node) -> bool {
    for accessor in nodea.accessors.iter() {
        match *accessor {
            Accessor::Ref(id) => {
                if nodeb.accessors.contains(&Accessor::Mut(id)) {
                    return true
                }
            },
            Accessor::Mut(id) => {
                if nodeb.accessors.contains(&Accessor::Mut(id)) || nodeb.accessors.contains(&Accessor::Ref(id)) {
                    return true
                }
            },
            Accessor::ResRef(id) => {
                if nodeb.accessors.contains(&Accessor::ResMut(id)) {
                    return true
                }
            },
            Accessor::ResMut(id) => {
                if nodeb.accessors.contains(&Accessor::ResMut(id)) || nodeb.accessors.contains(&Accessor::ResRef(id)) {
                    return true
                }
            }
        }
    }

    false
}