use crate::analyzer::taint::*;
use crate::tree::cursor::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Vertex<'a> {
    pub source: Taint,
    pub context: ContextStack,
    pub assign: Option<Taint>,
    pub path: Vec<Cursor<'a>>,
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
}

pub struct Graph<'a> {
    nodes: HashMap<Cursor<'a>, Vec<Vertex<'a>>>,
    // since we build working down, key is child and value is parents
    edges: HashMap<Cursor<'a>, Vec<Cursor<'a>>>,
    leaves: HashMap<Taint, Vec<Cursor<'a>>>,
}

impl<'a> Graph<'a> {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            leaves: HashMap::new(),
        }
    }

    pub fn dump(&self) -> String {
        let s = format!("digraph {{\n");
        for (k, v) in self.nodes.iter() {
            println!("\t[{}]\n", k.to_string())
        }
        for (k, v) in self.edges.iter() {}
        s
    }

    /// call this as you pop context so that next time you step
    /// into that block, you have fresh taints
    pub fn clear_scope(&mut self, scope: &Scope) {
        for (t, v) in self.leaves.iter_mut() {
            if t.scope == *scope {
                *v = Vec::new();
            }
        }
    }

    /// push a taint to the graph
    pub fn push(&mut self, cursor: Cursor<'a>, vertex: Vertex<'a>) {
        // if theres already a vertex at this node, add another
        if let Some(mut verts) = self.nodes.get_mut(&cursor) {
            verts.push(vertex.clone());
        } else {
            // otherwise insert
            self.nodes.insert(cursor.clone(), vec![vertex.clone()]);
        }

        // okay now we need to connect this to its parents
        self.add_edges(cursor.clone());

        // and finally, update our leaves
        if let Some(assign) = vertex.assign {
            if let Some(leaves) = self.leaves.get(&assign) {
                // the variable is already tainted, remove subtaints
                let mut newvec = vec![cursor.clone()];
                for leaf in leaves.iter() {
                    let leaf_ctx = &self.nodes.get(&leaf).unwrap().first().unwrap().context;
                    if !vertex.context.contains(&leaf_ctx) {
                        newvec.push(leaf.clone());
                    }
                }
                self.leaves.insert(assign, newvec);
            } else {
                // this is our first time tainting this variable, new insert
                self.leaves.insert(assign, vec![cursor.clone()]);
            }
        }
    }

    fn add_edges(&mut self, cursor: Cursor<'a>) {
        let vertex = self.nodes.get(&cursor).unwrap(); // we know this wont have an error since we just inserted to nodes
        let taint = &vertex.last().unwrap().source;
        if let Some(leaves) = self.leaves.get(&taint) {
            for leaf in leaves.iter() {
                if let Some(child) = self.edges.get_mut(&cursor) {
                    child.push(leaf.clone());
                } else {
                    self.edges.insert(cursor.clone(), vec![leaf.clone()]);
                }
            }
        } else {
            println!("WARNING: grapher is not playing well with analyzer :(")
        }
    }
}
