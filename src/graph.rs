use super::*;
use daggy::Dag;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone)]
struct Leaf {
    node: daggy::NodeIndex,
    context_stack: Vec<Context>,
}

pub struct Graph {
    dag: Dag<Vertex, Arc>,
    // last node that modified a taint
    leaves: HashMap<Taint, Vec<Leaf>>,
    params: HashMap<String, Vec<daggy::NodeIndex>>,
}

#[derive(Clone)]
pub enum Vertex {
    Source {
        tainting: Taint,
        context_stack: Vec<Context>,
    },
    Param {
        tainting: Taint,
        context_stack: Vec<Context>,
    },
    Assignment {
        kind: String,
        parent_taint: Taint,
        tainting: Taint,
        path: Vec<PathNode>,
        context_stack: Vec<Context>,
    },
    Resolved {
        parent_taint: Taint,
        name: String,
        path: Vec<PathNode>,
        context_stack: Vec<Context>,
    },
    Unresolved {
        parent_taint: Taint,
        name: String,
        path: Vec<PathNode>,
        context_stack: Vec<Context>,
    },
}

impl fmt::Debug for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        match self {
            Self::Param { tainting, .. } => {
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
            Self::Source { tainting, .. } => {
                s.push('$');
                s.push_str(&tainting.name);
            }
            Self::Assignment {
                parent_taint,
                tainting,
                path,
                ..
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

    pub fn push(&mut self, vertex: Vertex) {
        let id = self.dag.add_node(vertex.clone());

        match vertex {
            Vertex::Assignment {
                parent_taint,
                tainting,
                context_stack,
                ..
            } => {
                // add edges
                for leaf in self.leaves.get(&parent_taint).unwrap() {
                    _ = self.dag.add_edge(
                        leaf.node,
                        id,
                        Arc {
                            context_stack: context_stack.clone(),
                        },
                    );
                }

                if self.leaves.contains_key(&tainting) {
                    let leaf = self.leaves.get(&tainting).unwrap();
                    let mut newleaf = Vec::new();
                    for v in leaf.iter() {
                        if v.context_stack.len() > context_stack.len()
                            && &v.context_stack[0..context_stack.len()] == context_stack.as_slice()
                        {
                            // old leaf is subcontext, remove it
                            println!(
                                "prune old stack is longer and equal {:?} -> {:?}",
                                v.context_stack, context_stack
                            );
                        } else if v.context_stack == context_stack {
                            println!("prune old stack is equal");
                        } else {
                            newleaf.push(v.clone());
                        }
                    }
                    newleaf.push(Leaf {
                        node: id,
                        context_stack: context_stack,
                    });
                    self.leaves.insert(tainting, newleaf);
                } else {
                    self.leaves.insert(
                        tainting,
                        vec![Leaf {
                            node: id,
                            context_stack: context_stack,
                        }],
                    );
                }
            }
            Vertex::Param {
                tainting,
                context_stack,
                ..
            } => {
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
                    leaf.push(Leaf {
                        node: id,
                        context_stack: context_stack,
                    });
                } else {
                    self.leaves.insert(
                        tainting,
                        vec![Leaf {
                            node: id,
                            context_stack: context_stack,
                        }],
                    );
                }
            }
            Vertex::Source {
                tainting,
                context_stack,
                ..
            } => {
                if self.leaves.contains_key(&tainting) {
                    let leaf = self.leaves.get_mut(&tainting).unwrap();
                    leaf.push(Leaf {
                        node: id,
                        context_stack: context_stack,
                    });
                } else {
                    self.leaves.insert(
                        tainting,
                        vec![Leaf {
                            node: id,
                            context_stack: context_stack,
                        }],
                    );
                }
            }
            Vertex::Unresolved {
                parent_taint,
                context_stack,
                ..
            } => {
                // add edges
                for leaf in self.leaves.get(&parent_taint).unwrap() {
                    _ = self.dag.add_edge(
                        leaf.node,
                        id,
                        Arc {
                            context_stack: context_stack.clone(),
                        },
                    );
                }
            }
            Vertex::Resolved {
                parent_taint,
                name,
                context_stack,
                ..
            } => {
                // add edges
                for leaf in self.leaves.get(&parent_taint).unwrap() {
                    _ = self.dag.add_edge(
                        leaf.node,
                        id,
                        Arc {
                            context_stack: context_stack.clone(),
                        },
                    );
                }
                let params = self.params.get(&name);
                if let Some(params) = params {
                    for p in params {
                        _ = self.dag.add_edge(
                            id,
                            *p,
                            Arc {
                                context_stack: context_stack.clone(),
                            },
                        );
                    }
                }
            }
            _ => (),
        }
    }
}
