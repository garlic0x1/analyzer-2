use daggy::Dag;
use std::collections::HashMap;
use super::*;

pub struct Graph<'a> {
    dag: Dag<Vertex<'a>, Arc>,
    leaves: HashMap<&'a Taint<'a>, daggy::NodeIndex>,
}

#[derive(Clone)]
pub enum Vertex<'a> {
    Assignment {
        // type of assingment (assign, append, return, pass, etc)
        kind: String,
        // taint to create
        tainting: &'a Taint<'a>,
        // extra info
        code: String,
        position: Point,
        context_stack: Vec<Context>,
    },

    Resolved,
    Unresolved,
    Break,
}

pub struct Arc {
    // path of hooks, conditionals, and loops
    context_stack: Vec<Context>,
}

impl<'a> Graph<'a> {
    pub fn new() -> Self {
        Self { 
            dag: Dag::new(),
            leaves: HashMap::new(),
        } 
    }

    pub fn push(&mut self, vertex: Vertex<'a>, arc: Arc, parent_taint: &'a Taint<'a>) {
        let leaf = self.leaves.get(parent_taint);
        match leaf {
            Some(leaf) => {
                let id = self.dag.add_child(*leaf, arc, vertex.clone());
                match vertex {
                    Vertex::Assignment { tainting, .. } => {
                        self.leaves.insert(tainting, id.1);
                    },
                    _ => {
                        self.leaves.insert(parent_taint, id.1);
                    },
                }
            },
            None => {
                let id = self.dag.add_node(vertex.clone());
                match vertex {
                    Vertex::Assignment { tainting, .. } => {
                        self.leaves.insert(tainting, id);
                    },
                    _ => {
                        self.leaves.insert(parent_taint, id);
                    },
                }
            }
        }
    }
}
