use daggy::Dag;
use std::collections::HashMap;
use super::*;

pub struct Graph<'a> {
    dag: Dag<Vertex<'a>, Arc>,
    leaves: HashMap<Taint<'a>, daggy::NodeIndex>,
}

#[derive(Clone, Debug)]
pub enum Vertex<'a> {
    Assignment {
        // type of assingment (assign, append, return, pass, etc)
        kind: String,
        // taint to create
        tainting: Taint<'a>,
        //context_stack: Vec<Context>,
    },

    Resolved{
        name: String,
    },
    Unresolved{
        name: String,
    },
    Break{
        name: String,
    },
}

#[derive(Clone, Debug)]
pub struct Arc {
    // path of hooks, conditionals, and loops
    pub context_stack: Vec<Context>,
}

impl<'a> Graph<'a> {
    pub fn new() -> Self {
        Self { 
            dag: Dag::new(),
            leaves: HashMap::new(),
        } 
    }

    pub fn dump(&self) -> String {
        let dot = petgraph::dot::Dot::new(&self.dag);
        format!("{:?}", dot)
    }

    pub fn push(&mut self, vertex: Vertex<'a>, arc: Arc, parent_taint: Taint<'a>) {
        let leaf = self.leaves.get(&parent_taint);
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
