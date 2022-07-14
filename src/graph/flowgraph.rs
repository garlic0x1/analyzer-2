use crate::analyzer::taint::*;
use crate::tree::cursor::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Vertex<'a> {
    source: Taint,
    pub context: ContextStack,
    assign: Option<Taint>,
    pub path: Vec<Cursor<'a>>,
    pub parents: Vec<Cursor<'a>>,
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
            parents: Vec::new(),
        }
    }
}

impl<'a> std::fmt::Debug for Vertex<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.path.last().unwrap().to_string();

        write!(f, "{}", s)
    }
}

pub struct Graph<'a> {
    nodes: HashMap<Cursor<'a>, Vec<Vertex<'a>>>,
    leaves: HashMap<Taint, Vec<Cursor<'a>>>,
}

impl<'a> Graph<'a> {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            leaves: HashMap::new(),
        }
    }

    pub fn dump(&self) -> String {
        let mut s = format!("digraph {{\n");
        let mut i = 0;
        for (k, v) in self.nodes.iter() {
            s.push_str(&format!(
                "\t{i} [ label = \"{}\" ]\n",
                k.to_string().replace("\"", "\\\"")
            ));
            for vert in v.iter() {
                for c in vert.parents.iter() {
                    let mut j = 0;
                    for (k, _) in self.nodes.iter() {
                        if c == k {
                            s.push_str(&format!("\t{j} -> {i} []\n"));
                        }
                        j += 1;
                    }
                }
            }
            i += 1;
        }
        s.push('}');
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

    // get rid of returns after using them, since they have global scope
    pub fn clear_returns(&mut self) {
        for (t, v) in self.leaves.iter_mut() {
            if t.kind == TaintKind::Return {
                *v = Vec::new();
            }
        }
    }

    /// walk up a graph from vertex key
    pub fn walk(&self) -> Vec<Vec<Vertex<'a>>> {
        let mut paths = Vec::new();
        for (k, v) in self.nodes.iter() {
            if let None = v.last().unwrap().assign {
                let stack: Vec<Vertex> = vec![v.last().unwrap().clone()];
                let new = self.depth_first(stack.clone());
                if true || new.len() > 0 {
                    paths.extend(new);
                }
            }
        }

        paths
    }

    /// recursively search for paths
    fn depth_first(&self, stack: Vec<Vertex<'a>>) -> Vec<Vec<Vertex<'a>>> {
        let mut stacks = Vec::new();
        let mut stack = stack.clone();
        if let Some(vert) = stack.clone().last() {
            let mut counter = 0;
            for parent in vert.parents.iter() {
                stack.push(self.nodes.get(&parent).unwrap().last().unwrap().clone());
                stacks.extend(self.depth_first(stack.clone()));
                stack.pop();
                counter += 1;
            }
            if counter == 0 && stack.len() > 0 {
                stacks.push(stack.clone());
            }
        }
        stacks
    }

    /// push a taint to the graph
    pub fn push(&mut self, cursor: Cursor<'a>, vertex: Vertex<'a>) {
        // if theres already a vertex at this node, add another
        if let Some(verts) = self.nodes.get_mut(&cursor) {
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
        let vertex = self.nodes.get_mut(&cursor).unwrap().last_mut().unwrap(); // we know this wont have an error since we just inserted to nodes
        let taint = &vertex.source;

        if let Some(leaves) = self.leaves.get(&taint) {
            for leaf in leaves.iter() {
                vertex.parents.push(leaf.clone());
            }
        } else {
            // this just happens on sources, we can polish later
            //println!("{:?}", taint);
        }
    }
}
