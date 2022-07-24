use crate::analyzer::taint::*;
use crate::tree::cursor::*;
use std::collections::HashMap;
use std::collections::HashSet;

use super::rules::Rules;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct PathItem<'a> {
    pub source: Taint,
    pub path: Vec<Cursor<'a>>,
}

impl<'a> PathItem<'a> {
    pub fn new(source: Taint, path: Vec<Cursor<'a>>) -> Self {
        Self { source, path }
    }

    pub fn to_str(&self) -> &str {
        match self.path.first() {
            Some(cur) => cur.to_str(),
            None => "",
        }
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
        let mut s = String::new(); // self.paths.iter().next().unwrap().0.to_str().to_string();

        let paths = self.paths.iter().next().unwrap().0;

        for seg in paths.path.iter().rev() {
            s.push_str(seg.kind());
            s.push_str(" <- ");
        }
        s.push_str(&paths.source.name);

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
                (&format!("{} [{:?}]", k.to_str(), v)).replace("\"", "\\\"")
            ));
            if let Some((path, _parent)) = v.paths.iter().next() {
                for c in path.path.iter() {
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

    pub fn verts_to_path(&'a self, vert_path: Vec<Cursor<'a>>) -> Vec<Cursor<'a>> {
        let mut last_cur: Option<Cursor<'a>> = None;
        let mut out_path = Vec::new();
        for vert in vert_path.iter() {
            if let Some(vert_cur) = last_cur {
                let last = self.nodes.get(&vert_cur).unwrap();
                for (path, parents) in last.paths().iter() {
                    if parents.contains(vert) {
                        out_path.extend(path.path.clone());
                        break;
                    }
                }
            }
            last_cur = Some(vert.clone());
        }

        // if let Some(vert) = self.nodes.get(&vert_path.last().unwrap()) {
        //     for (item, path) in vert.paths.iter() {}
        // }

        out_path
    }

    pub fn crawl_sinks(&'a self, ruleset: &Rules) -> Vec<Vec<Cursor<'a>>> {
        let mut paths = Vec::new();
        for (k, v) in self.nodes.iter() {
            for (path, parent) in v.paths().iter() {
                for cur in path.path.iter() {
                    let mut crawl = ruleset.sinks().contains_key(&k.name().unwrap_or_default());
                    if ruleset.sinks().contains_key(k.kind()) {
                        crawl = true;
                    }
                    if let Some(n) = cur.name() {
                        //println!("{:?}", ruleset.sinks());
                        if ruleset.sinks().contains_key(&n) {
                            crawl = true;
                        }
                    }
                    if ruleset.sinks().contains_key(cur.kind()) {
                        crawl = true;
                    }
                    if crawl {
                        let mut stack: Vec<Cursor> = vec![k.clone()];
                        for (_, parents) in v.paths().iter() {
                            for parent in parents.iter() {
                                stack.push(parent.clone());
                                let new = self.defi_verts(&stack, parent);
                                paths.extend(new);
                            }
                        }
                    }
                }
                if let None = v.assign {}
            }
        }
        paths
    }

    fn defi_verts(&'a self, stack: &Vec<Cursor<'a>>, last_cur: &'a Cursor) -> Vec<Vec<Cursor<'a>>> {
        let mut stacks = Vec::new();
        let mut stack = stack.clone();
        let last_vert = self.nodes.get(&last_cur).unwrap();
        let mut counter = 0;
        for (_, parents) in last_vert.paths().iter() {
            for parent in parents.iter() {
                if stack.contains(parent) {
                    continue;
                }
                stack.push(parent.clone());
                stacks.extend(self.defi_verts(&stack, parent));
                stack.pop();
                counter += 1;
            }
            if counter == 0 && stack.len() > 0 {
                stacks.push(stack.clone());
            }
        }
        stacks
    }

    /*
    pub fn oracle(&self, ruleset: &Rules, vertices: &Vec<Cursor>) -> bool {
        let mut last_vert: Option<Vertex> = None;
        for vert_cur in vertices {
            let vert = self.nodes.get(vert_cur).unwrap();
            for (item, path) in vert.paths() {}
        }
    }
    */

    pub fn test_path(&self, ruleset: &Rules, path: &Vec<Cursor>) -> bool {
        for segment in path.iter() {
            let segname = &segment.name().unwrap_or_default();
            let segkind = segment.kind();
            for (_kind, vuln) in ruleset.vulns().iter() {
                let mut cont = false;
                if vuln.sinks.contains_key(segname) {
                    cont = true;
                } else if vuln.sinks.contains_key(segkind) {
                    cont = true;
                }

                if cont {
                    for segment in path.iter() {
                        let vert = self.nodes.get(segment).unwrap();
                        for (path, _parent) in vert.paths.iter() {
                            for segment in path.path.iter() {
                                let segname = &segment.name().unwrap_or_default();
                                let segkind = segment.kind();
                                if vuln.sanitizers.contains_key(segname)
                                    || vuln.sanitizers.contains_key(segkind)
                                {
                                    return false;
                                }
                            }
                            if vuln.sources.contains(&path.source.name) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
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
