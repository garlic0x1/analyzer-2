use super::*;
use crate::analyzer::*;
use daggy::Dag;
use std::collections::HashMap;
use std::fmt;

pub struct Graph {
    dag: Dag<Vertex, Arc>,
    // last node that modified a taint
    leaves: HashMap<Taint, daggy::NodeIndex>,
    params: HashMap<String, Vec<daggy::NodeIndex>>,
}

#[derive(Clone)]
pub enum Vertex {
    Source {
        tainting: Taint,
    },
    Param {
        tainting: Taint,
    },
    Assignment {
        kind: String,
        parent_taint: Taint,
        tainting: Taint,
        path: Vec<PathNode>,
    },
    Resolved {
        parent_taint: Taint,
        name: String,
        path: Vec<PathNode>,
    },
    Unresolved {
        parent_taint: Taint,
        name: String,
        path: Vec<PathNode>,
    },
}

impl fmt::Debug for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        match self {
            Self::Param { tainting } => {
                s.push('$');
                s.push_str(&tainting.name);
            }
            Self::Resolved {
                parent_taint,
                name,
                path,
            } => {
                for n in path.iter().rev() {
                    s.push_str(format!("{:?} <- ", n).as_str());
                }
                s.push_str(format!("{}", parent_taint.name).as_str());
            }
            Self::Source { tainting } => {
                s.push('$');
                s.push_str(&tainting.name);
            }
            Self::Assignment {
                parent_taint,
                kind,
                tainting,
                path,
            } => {
                s.push_str(format!("[{}] {}", kind, tainting.name).as_str());
                for n in path.iter().rev() {
                    s.push_str(format!(" <- {:?}", n).as_str());
                }
                s.push_str(format!(" <- {}", parent_taint.name).as_str());
            }
            Self::Unresolved {
                parent_taint,
                name,
                path,
            } => {
                for n in path.iter().rev() {
                    s.push_str(format!("{:?} <- ", n).as_str());
                }
                s.push_str(format!("{}", parent_taint.name).as_str());
            }
        }
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug)]
pub enum PathNode {
    Resolved { name: String },
    Unresolved { name: String },
}

#[derive(Clone, Debug)]
pub struct Arc {
    // path of hooks, conditionals, and loops
    pub context_stack: Vec<Context>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            dag: Dag::new(),
            leaves: HashMap::new(),
            params: HashMap::new(),
        }
    }

    pub fn dump(&self) -> String {
        let dot = petgraph::dot::Dot::new(&self.dag);
        format!("{:?}", dot)
    }

    pub fn push(&mut self, vertex: Vertex, arc: Option<Arc>, parent_taint: Option<Taint>) {
        
        if let Some(parent) = parent_taint {
            let arc = arc.unwrap();
            let debug = parent.name.clone();
            let leaf = self
                .leaves
                .get(&parent)
                .expect(&format!("no parent found {:?}", vertex.clone()));
            let id = self
                .dag
                .add_child(*leaf, arc.clone(), vertex.clone());

            match vertex {
                Vertex::Assignment { kind, parent_taint, tainting, path } => {
                    self.leaves.insert(tainting, id.1);
                }
                Vertex::Resolved { parent_taint, name, path } => {
                    let params = self.params.get(&name);
                    if let Some(params) = params {
                        for p in params {
                            self.dag.add_edge(id.1, *p, arc.clone());
                        }
                    }
                }
                _ => ()
            }
        } else {
            let id = self.dag.add_node(vertex.clone());
            match vertex {
                Vertex::Param { tainting } => {
                    let name = tainting.clone().scope.function.unwrap();
                    if self.params.contains_key(&name) {
                        let param = self.params.get_mut(&tainting.scope.clone().function.unwrap()).unwrap();
                        param.push(id);
                    } else {
                        self.params.insert(tainting.scope.clone().function.unwrap(), vec![id]);
                    }
                    self.leaves.insert(tainting, id);
                }
                Vertex::Source { tainting, .. } => {
                    self.leaves.insert(tainting, id);
                }
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
