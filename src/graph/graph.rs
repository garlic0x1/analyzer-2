use super::rules::{Rules, VertKind, Vuln};
use super::vertex::*;
use crate::analyzer::taint::*;
use crate::tree::cursor::*;
use crate::tree::resolved::Resolved;
use std::collections::{HashMap, HashSet};

pub struct Graph<'a> {
    nodes: HashMap<Cursor<'a>, Vertex<'a>>,
    leaves: HashMap<Taint, Vec<Cursor<'a>>>,
    returns: HashMap<Resolved<'a>, Vec<Cursor<'a>>>,
}

impl<'a> Graph<'a> {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            leaves: HashMap::new(),
            returns: HashMap::new(),
        }
    }

    pub fn returns(&self, resolved: &Resolved<'a>) -> Option<&Vec<Cursor<'a>>> {
        self.returns.get(resolved)
    }

    /// push a taint to the graph, returns false if recursive
    pub fn push(&mut self, path: PathItem<'a>, cursor: Cursor<'a>, vertex: Vertex<'a>) -> bool {
        let mut vertex = vertex;
        let known = self.nodes.contains_key(&cursor);

        match known {
            true => {
                self.add_edges(path, &mut vertex);
                self.update_leaves(cursor, vertex)
            }
            false => {
                self.add_edges(path, &mut vertex);
                self.nodes.insert(cursor.clone(), vertex.clone());
                self.update_leaves(cursor, vertex)
            }
        }

        !known
    }

    pub fn match_rules(&self, ruleset: &Rules) -> HashSet<Vec<Cursor>> {
        let mut results = HashSet::new();

        for (k, v) in self.nodes.iter() {
            for (_, path) in v.parents().iter().chain(v.sources().iter()) {
                for segment in path.segments() {
                    let name = segment.name().unwrap_or_default();
                    let kind = segment.kind().to_string();
                    for (_, vuln) in ruleset.vulns().iter() {
                        if vuln.has_sink(&name) || vuln.has_sink(&kind) {
                            results.extend(self.crawl(&vuln, vec![k.clone()]));
                        }
                    }
                }
            }
        }

        results
    }

    /// dump graph into DOT format string
    pub fn dump(&self) -> String {
        let mut s = format!("digraph {{\n");
        let mut i = 0;
        for (k, v) in self.nodes.iter() {
            s.push_str(&format!(
                "\t{i} [ label = \"{}\" ]\n",
                k.to_str().replace("\"", "\\\"")
            ));

            for (parent, _path) in v.parents().iter() {
                let mut j = 0;
                for (k, _) in self.nodes.iter() {
                    if parent == k {
                        s.push_str(&format!("\t{j} -> {i} []\n"));
                    }
                    j += 1;
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

    /// get rid of returns after using them, since they have global scope

    //
    // NEED TO IMPROVE THIS, SECOND CALL OF FUNC DOESNT RETURN TAINT
    //
    pub fn clear_returns(&mut self) {
        for (t, v) in self.leaves.iter_mut() {
            if t.kind == TaintKind::Return {
                *v = Vec::new();
            }
        }
    }

    /// connect a given vertex to the leaves
    fn add_edges(&mut self, path: PathItem<'a>, vertex: &mut Vertex<'a>) {
        if let Some(leaves) = self.leaves.get(&path.source()) {
            for leaf in leaves {
                vertex.add_parent(leaf.clone(), path.clone());
            }
        } else {
            // need to create a parent to the source
            if let Some(source_cur) = path.segments().last() {
                vertex.add_source(source_cur.clone(), path);
            }
        }
    }

    /// add assigning leaf, and prune as needed
    fn update_leaves(&mut self, cursor: Cursor<'a>, vertex: Vertex<'a>) {
        if let Some(assign) = vertex.assign() {
            if let Some(leaves) = self.leaves.get(&assign) {
                // variabe is already tainted, remove overwritten taints
                let mut newvec = vec![cursor];
                for leaf in leaves.iter() {
                    if let Some(vert) = self.nodes.get(&leaf) {
                        if !vertex.context().contains(vert.context()) {
                            newvec.push(leaf.clone());
                        }
                    }
                }
                self.leaves.insert(assign.clone(), newvec);
            } else {
                // first time this taint has occured
                self.leaves.insert(assign.clone(), vec![cursor]);
            }
        }
    }

    fn crawl(&self, vuln: &Vuln, stack: Vec<Cursor<'a>>) -> HashSet<Vec<Cursor>> {
        let mut results = HashSet::new();
        let mut stack = stack.clone();

        if let Some(last) = stack.last() {
            if let Some(vert) = self.nodes.get(last) {
                // crawl up each parent taint
                for (parent, path) in vert.parents().iter() {
                    let mut sanitized = false;
                    for segment in path.segments() {
                        match vuln.identify(segment.clone()) {
                            Some(VertKind::Sanitizer) => {
                                sanitized = true;
                                break;
                            }
                            Some(VertKind::Source) => {
                                results.insert(stack.clone());
                            }
                            _ => {}
                        }
                    }
                    if !sanitized {
                        if vuln.has_source(&path.source().name) {
                            results.insert(stack.clone());
                        }
                        stack.push(parent.clone());
                        results.extend(self.crawl(vuln, stack.clone()));
                        stack.pop();
                    }
                }
                // crawl up each source taint
                for (_, path) in vert.sources().iter() {
                    let mut sanitized = false;
                    for segment in path.segments() {
                        match vuln.identify(segment.clone()) {
                            Some(VertKind::Sanitizer) => {
                                sanitized = true;
                                break;
                            }
                            Some(VertKind::Source) => {
                                results.insert(stack.clone());
                            }
                            _ => {}
                        }
                    }

                    if !sanitized && vuln.has_source(&path.source().name) {
                        results.insert(stack.clone());
                    }
                }
            }
        }

        results
    }
}
