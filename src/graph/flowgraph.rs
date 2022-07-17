use crate::analyzer::taint::*;
use crate::tree::cursor::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

// #[derive(Clone)]
// pub struct Veretex<'a> {
//     source: Taint,
//     pub context: ContextStack,
//     assign: Option<Taint>,
//     pub path: Vec<Cursor<'a>>,
//     pub parents: Vec<Cursor<'a>>,
// }

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct PathItem<'a> {
    pub source: Taint,
    pub path: Vec<Cursor<'a>>,
}

impl<'a> PathItem<'a> {
    pub fn new(source: Taint, path: Vec<Cursor<'a>>) -> Self {
        Self { source, path }
    }
}

#[derive(Clone)]
pub struct Vertex<'a> {
    assign: Option<Taint>,
    context: ContextStack,
    // map paths to parents
    paths: HashMap<PathItem<'a>, HashSet<Cursor<'a>>>,
}

impl<'a> Vertex<'a> {
    pub fn new(context: ContextStack, assign: Option<Taint>, path: PathItem<'a>) -> Self {
        let mut paths = HashMap::new();
        paths.insert(path, HashSet::new());
        Self {
            context,
            assign,
            paths,
        }
    }

    pub fn push(&mut self, path_item: PathItem<'a>, parents: HashSet<Cursor<'a>>) {
        self.paths.insert(path_item, parents);
    }

    pub fn paths(&self) -> &HashMap<PathItem<'a>, HashSet<Cursor<'a>>> {
        &self.paths
    }
}

impl<'a> std::fmt::Debug for Vertex<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .paths
            .iter()
            .next()
            .unwrap()
            .1
            .iter()
            .next()
            .unwrap()
            .to_string();

        write!(f, "{}", s)
    }
}

pub struct Graph<'a> {
    nodes: HashMap<Cursor<'a>, Vertex<'a>>,
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
            for (path, parents) in v.paths.iter() {
                for c in parents.iter() {
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
    pub fn walk(&'a self) -> Vec<Vec<Cursor<'a>>> {
        let mut paths = Vec::new();
        for (_, v) in self.nodes.iter() {
            if let None = v.assign {
                for (path, parents) in v.paths().iter() {
                    let stack: Vec<Cursor> = path.path.clone();
                    let new = self.depth_first(&stack, v);
                    if true || new.len() > 0 {
                        paths.extend(new);
                    }
                }
            }
        }

        paths
    }

    /// recursively search for paths
    fn depth_first(
        &'a self,
        stack: &Vec<Cursor<'a>>,
        last_vert: &'a Vertex,
    ) -> Vec<Vec<Cursor<'a>>> {
        let mut stacks = Vec::new();
        let mut stack = stack.clone();
        let mut counter = 0;
        for (path, parents) in last_vert.paths().iter() {
            for parent in parents.iter() {
                if stack.contains(parent) {
                    continue;
                }
                let parent_vert = self.nodes.get(parent).expect("no such parent");
                stack.extend(path.path.clone());
                stacks.extend(self.depth_first(&stack, parent_vert));
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
    /// returns true if the vertex is unknown
    /// return false if known to stop crawl
    pub fn push(&mut self, cursor: Cursor<'a>, vertex: Vertex<'a>) -> bool {
        let mut known = false;
        // if theres already a vertex at this node, add another
        // NO. BAD IDEA
        // add another path, much nicer
        if let Some(vert) = self.nodes.get_mut(&cursor) {
            // verts.push(vertex.clone());
            for (path, parents) in vertex.paths.iter() {
                vert.push(path.clone(), parents.clone());
            }
            known = true;
        } else {
            // otherwise insert
            self.nodes.insert(cursor.clone(), vertex.clone());
        }

        // okay now we need to connect this to its parents
        self.add_edges(cursor.clone());

        // and finally, update our leaves
        if let Some(assign) = vertex.assign {
            if let Some(leaves) = self.leaves.get(&assign) {
                // the variable is already tainted, remove subtaints
                let mut newvec = vec![cursor.clone()];
                for leaf in leaves.iter() {
                    let leaf_ctx = &self.nodes.get(&leaf).unwrap().context;
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
        !known
    }

    fn add_edges(&mut self, cursor: Cursor<'a>) {
        let vertex = self.nodes.get_mut(&cursor).unwrap(); // we know this wont have an error since we just inserted to nodes
        for (path, parents) in vertex.paths.iter_mut() {
            let taint = &path.source;

            if let Some(leaves) = self.leaves.get(&taint) {
                for leaf in leaves.iter() {
                    if leaf != &cursor {
                        parents.insert(leaf.clone());
                    }
                }
            } else {
                // this just happens on sources, we can polish later
                //println!("{:?}", taint);
            }
        }
    }
}
