use crate::analyzer::taint::*;
use crate::tree::cursor::*;
use daggy::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Vertex<'a> {
    pub source: Taint,
    pub context: ContextStack,
    pub assign: Option<Taint>,
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
        if let Some(t) = &self.assign {
            s.push_str(" -> ");
            s.push_str(&t.name);
        }
        write!(f, "{}", s)
    }
}

impl<'a> Vertex<'a> {
    pub fn new(
        source: Taint,
        context: ContextStack,
        assign: Option<Taint>,
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
            assign: None,
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

    pub fn push(&mut self, vertex: Vertex<'a>) {
        let id = self.dag.add_node(vertex.clone());
        self.connect_parents(&id, &vertex);

        match vertex.assign {
            // modify leaves
            Some(assign) => {}
            // dont modify leaves
            None => {}
        }
    }

    fn connect_parents(&mut self, id: &NodeIndex, vertex: &Vertex) {
        match vertex.source.kind.as_str() {
            // add source
            "global" => {
                if !self.taint_leaves.contains_key(&vertex.source) {
                    let new_leaf = Vertex::new_source(vertex.source.clone());
                    let leaf_id = self.dag.add_node(new_leaf);
                    let mut new_map = HashMap::new();
                    new_map.insert(ContextStack::new(), leaf_id);
                    self.taint_leaves.insert(vertex.source.clone(), new_map);
                }
                for (_, leaf) in self.taint_leaves.get(&vertex.source).unwrap().iter() {
                    _ = self
                        .dag
                        .add_edge(leaf.clone(), id.clone(), ContextStack::new());
                }
            }
            _ => {}
        }
    }
}
