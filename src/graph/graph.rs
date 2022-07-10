use crate::analyzer::taint::*;
use crate::tree::cursor::*;
use daggy::*;
use std::collections::HashMap;

#[derive(Clone)]
pub enum Assign<'a> {
    Taint { assign: Taint },
    Unresolved { cursor: Cursor<'a> },
}

impl<'a> std::fmt::Debug for Assign<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        match self {
            Assign::Taint { assign } => {
                s.push_str(&format!("{:?}", assign));
            }
            Assign::Unresolved { cursor } => {
                s.push_str(&format!("{:?}", cursor.name()));
            }
        }
        write!(f, "{}", s)
    }
}

#[derive(Clone)]
pub struct Vertex<'a> {
    pub source: Taint,
    pub context: ContextStack,
    pub assign: Assign<'a>,
    pub path: Vec<Cursor<'a>>,
}

impl<'a> std::fmt::Debug for Vertex<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s.push_str(&self.source.name);
        for item in self.path.iter() {
            s.push_str(" -> ");
            s.push_str(&item.name().unwrap());
        }

        s.push_str(&format!("{:?}", self.assign));

        // if let Some(t) = &self.assign {
        //     s.push_str(" -> ");
        //     s.push_str(&t.name);
        // }
        write!(f, "{}", s)
    }
}

impl<'a> Vertex<'a> {
    pub fn new(
        source: Taint,
        context: ContextStack,
        assign: Assign<'a>,
        path: Vec<Cursor<'a>>,
    ) -> Self {
        Self {
            source,
            context,
            assign,
            path,
        }
    }

    pub fn new_source(source: Taint) -> Self {
        Self {
            source: source.clone(),
            context: ContextStack::new(),
            assign: Assign::Taint { assign: source },
            path: Vec::new(),
        }
    }
    pub fn new_param(source: Taint, assign: Assign<'a>) -> Self {
        Self {
            source,
            context: ContextStack::new(),
            assign,
            path: Vec::new(),
        }
    }
}

pub struct Graph<'a> {
    dag: Dag<Vertex<'a>, ContextStack>,
    taint_leaves: HashMap<Taint, HashMap<ContextStack, NodeIndex>>,
}

impl<'a> Graph<'a> {
    pub fn new() -> Self {
        Self {
            dag: Dag::new(),
            taint_leaves: HashMap::new(),
        }
    }

    pub fn dump(&self) -> String {
        let dot = petgraph::dot::Dot::new(&self.dag);
        format!("{:?}", dot)
    }

    pub fn clear_taint(&mut self, taint: &Taint) {
        self.taint_leaves.remove(taint);
    }

    pub fn clear_scope(&mut self, scope: &Scope) {
        for (t, l) in self.taint_leaves.iter_mut() {
            if t.scope == *scope {
                *l = HashMap::new();
            }
        }
    }

    pub fn push(&mut self, vertex: Vertex<'a>) {
        let id = self.dag.add_node(vertex.clone());
        self.connect_parents(&id, &vertex);

        match vertex.clone().assign {
            // modify leaves
            Assign::Taint { assign } => {
                println!("assign{:?}", vertex.clone());
                if let Some(leaves) = self.taint_leaves.get_mut(&assign) {
                    for (ctx, _) in leaves.clone().iter() {
                        if vertex.context.contains(&ctx) {
                            leaves.remove(&ctx);
                        }
                    }
                    leaves.insert(vertex.context, id);
                } else {
                    let mut newmap = HashMap::new();
                    newmap.insert(vertex.context, id);
                    self.taint_leaves.insert(assign, newmap);
                }
            }
            // dont modify leaves
            Assign::Unresolved { cursor } => {}
        }
    }

    fn connect_parents(&mut self, id: &NodeIndex, vertex: &Vertex<'a>) {
        match &vertex.assign {
            Assign::Taint { assign } => match assign.kind {
                // add source
                TaintKind::Variable | TaintKind::Source | TaintKind::Param => {
                    if !self.taint_leaves.contains_key(&vertex.source.clone()) {
                        let new_leaf =
                            Vertex::new_param(vertex.source.clone(), vertex.assign.clone());
                        let leaf_id = self.dag.add_node(new_leaf);
                        let mut new_map = HashMap::new();
                        new_map.insert(vertex.context.clone(), leaf_id);
                        println!("inserted leaf");
                        self.taint_leaves.insert(vertex.source.clone(), new_map);
                    }
                    for (_, leaf) in self.taint_leaves.get(&vertex.source).unwrap().iter() {
                        _ = self
                            .dag
                            .add_edge(leaf.clone(), id.clone(), vertex.context.clone());
                    }
                }
                _ => (),
            },
            // shouldnt need to add source, so assume there are leaves but still handle errors
            Assign::Unresolved { cursor } => {
                if let Some(leaves) = self.taint_leaves.get(&vertex.source) {
                    for (_, leaf) in leaves.iter() {
                        _ = self
                            .dag
                            .add_edge(leaf.clone(), id.clone(), vertex.context.clone());
                    }
                } else {
                    println!("unexpected behavior",);
                }
            }
        }
    }
}
