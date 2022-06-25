use super::*;
use crate::analyzer::*;
use daggy::Dag;
use std::collections::HashMap;
use std::fmt;

pub struct Graph<'a> {
    dag: Dag<Vertex<'a>, Arc>,
    // last node that modified a taint
    leaves: HashMap<Taint<'a>, daggy::NodeIndex>,
}

#[derive(Clone)]
pub enum Vertex<'a> {
    Assignment {
        kind: String,
        parent_taint: Taint<'a>,
        tainting: Taint<'a>,
        path: Vec<PathNode>,
    },
    Unresolved {
        parent_taint: Taint<'a>,
        name: String,
        path: Vec<PathNode>,
    },
}

impl<'a> fmt::Debug for Vertex<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        match self {
            Self::Assignment {
                parent_taint,
                kind,
                tainting,
                path,
            } => {
                s.push_str(format!("[{}] {}", kind, tainting.name).as_str());
                for n in path.iter().rev() {
                    s.push_str(format!(" <- {}", n.name).as_str());
                }
                s.push_str(format!(" <- {}", parent_taint.name).as_str());
            }
            Self::Unresolved {
                parent_taint,
                name,
                path,
            } => {
                for n in path.iter().rev() {
                    s.push_str(format!("{} <- ", n.name).as_str());
                }
                s.push_str(format!("{}", parent_taint.name).as_str());
            }
        }
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug)]
pub struct PathNode {
    pub name: String,
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
                if let Vertex::Assignment {
                    parent_taint,
                    tainting,
                    ..
                } = vertex
                {
                    self.leaves.insert(tainting, id.1);
                }
            }
            None => {
                let id = self.dag.add_node(vertex.clone());
                match vertex {
                    Vertex::Assignment { tainting, .. } => {
                        self.leaves.insert(tainting, id);
                    }
                    _ => {
                        //self.leaves.insert(parent_taint, id);
                    }
                }
            }
        }
    }
}
