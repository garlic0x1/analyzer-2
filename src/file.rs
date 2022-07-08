use std::collections::HashMap;
use tree_sitter::*;

pub struct File<'a> {
    name: String,
    source: &'a str,
    tree: &'a Tree,
    resolved: HashMap<String, TreeCursor<'a>>,
}

impl<'a> File<'a> {
    pub fn new(name: String, tree: &'a Tree, source: &'a str) -> Self {
        Self {
            name,
            source,
            tree,
            resolved: HashMap::new(),
        }
    }

    pub fn push_resolved(&mut self, name: String, cur: TreeCursor<'a>) {
        self.resolved.insert(name, cur);
    }

    pub fn get_source(&self) -> &'a str {
        self.source
    }
}
