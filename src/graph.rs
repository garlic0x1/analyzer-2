use super::*;
use daggy::Dag;
use std::collections::HashMap;
use std::fmt;

pub struct Graph {
    dag: Dag<Vertex, Arc>,
    // last node that modified a taint
    leaves: HashMap<Taint, Vec<daggy::NodeIndex>>,
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
                parent_taint, path, ..
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
                s.push_str(format!("Assign {}", tainting.name).as_str());
                for n in path.iter().rev() {
                    s.push_str(format!(" <- {:?}", n).as_str());
                }
                s.push_str(format!(" <- {}", parent_taint.name).as_str());
            }
            Self::Unresolved {
                parent_taint, path, ..
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

impl fmt::Debug for Arc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(s) = self.context_stack.get(0) {
            write!(f, "{}", s.kind)
        } else {
            write!(f, "")
        }
    }
}

#[derive(Clone, Debug)]
pub enum PathNode {
    Resolved { name: String },
    Unresolved { name: String },
}

#[derive(Clone)]
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

    pub fn push(&mut self, vertex: Vertex, arc: Option<Arc>) {
        if let Some(arc) = arc {
            let id = self.dag.add_node(vertex.clone());

            match vertex {
                Vertex::Assignment {
                    parent_taint,
                    tainting,
                    ..
                } => {
                    // add edges
                    for leaf in self.leaves.get(&parent_taint).unwrap() {
                        self.dag.add_edge(*leaf, id, arc.clone());
                    }

                    if self.leaves.contains_key(&tainting) {
                        let leaf = self.leaves.get_mut(&tainting).unwrap();
                        leaf.push(id);
                    } else {
                        self.leaves.insert(tainting, vec![id]);
                    }
                }
                Vertex::Unresolved { parent_taint, .. } => {
                    // add edges
                    for leaf in self.leaves.get(&parent_taint).unwrap() {
                        self.dag.add_edge(*leaf, id, arc.clone());
                    }
                }
                Vertex::Resolved {
                    parent_taint, name, ..
                } => {
                    // add edges
                    for leaf in self.leaves.get(&parent_taint).unwrap() {
                        self.dag.add_edge(*leaf, id, arc.clone());
                    }
                    let params = self.params.get(&name);
                    if let Some(params) = params {
                        for p in params {
                            self.dag.add_edge(id, *p, arc.clone());
                        }
                    }
                }
                _ => (),
            }
        } else {
            let id = self.dag.add_node(vertex.clone());
            match vertex {
                Vertex::Param { tainting } => {
                    let name = tainting.clone().scope.function.unwrap();
                    if self.params.contains_key(&name) {
                        let param = self
                            .params
                            .get_mut(&tainting.scope.clone().function.unwrap())
                            .unwrap();
                        param.push(id);
                    } else {
                        self.params
                            .insert(tainting.scope.clone().function.unwrap(), vec![id]);
                    }
                    if self.leaves.contains_key(&tainting) {
                        let leaf = self.leaves.get_mut(&tainting).unwrap();
                        leaf.push(id);
                    } else {
                        self.leaves.insert(tainting, vec![id]);
                    }
                }
                Vertex::Source { tainting, .. } => {
                    if self.leaves.contains_key(&tainting) {
                        let leaf = self.leaves.get_mut(&tainting).unwrap();
                        leaf.push(id);
                    } else {
                        self.leaves.insert(tainting, vec![id]);
                    }
                }
                Vertex::Assignment { tainting, .. } => {
                    if self.leaves.contains_key(&tainting) {
                        let leaf = self.leaves.get_mut(&tainting).unwrap();
                        leaf.push(id);
                    } else {
                        self.leaves.insert(tainting, vec![id]);
                    }
                }
                _ => {
                    //self.leaves.insert(parent_taint, id);
                }
            }
        }
    }
}
