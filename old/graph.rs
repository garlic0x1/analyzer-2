use crate::analyzer::taint::*;
use daggy::Dag;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
struct Leaf {
    node: daggy::NodeIndex,
    context_stack: Vec<Context>,
}

pub struct Graph {
    dag: Dag<Vertex, Arc>,
    // last node that modified a taint
    leaves: HashMap<Taint, Vec<Leaf>>,
    last_resolved: Option<daggy::NodeIndex>,
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
    Return {
        parent_taint: Taint,
        tainting: Taint,
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
            Self::Return {
                parent_taint,
                tainting,
                path,
                context_stack,
            } => {
                s.push_str(format!("Return ").as_str());
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
            last_resolved: None,
        }
    }

    pub fn dump(&self) -> String {
        let dot = petgraph::dot::Dot::new(&self.dag);
        format!("{:?}", dot)
    }

    pub fn has_return(&self, taint: &Taint) -> bool {
        self.leaves.contains_key(taint)
    }

    pub fn clear_return(&mut self, taint: &Taint) {
        if taint.kind == "return" {
            self.leaves.remove(taint);
        }
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
                    println!("{:?}", leaf.clone());
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
                self.dag.add_edge(
                    self.last_resolved.expect("no resolved?"),
                    id,
                    Arc {
                        context_stack: context_stack.clone(),
                    },
                );
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
                self.last_resolved = Some(id.clone());
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
            Vertex::Return {
                parent_taint,
                tainting,
                path,
                context_stack,
            } => {
                println!("pushing Return vertex to graph {:?}", tainting.clone());
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
            _ => (),
        }
    }
}
