use std::collections::HashMap;

use super::{cursor::Cursor, resolved::Resolved};
use tree_sitter::*;

pub struct File<'a> {
    name: String,
    source: &'a str,
    tree: &'a Tree,
}

impl<'a> File<'a> {
    pub fn new(name: String, tree: &'a Tree, source: &'a str) -> Self {
        Self { name, source, tree }
    }

    /* resolver stuff
    pub fn contains(&self, name: &str) -> bool {
        self.resolved.contains(name)
    }

    pub fn get_resolved(&self, name: &str) -> Option<Resolved> {
        self.resolved.get(name)
    }
    */
    fn resolve(&mut self) -> HashMap<String, Resolved> {
        self.cursor().resolve()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn get_source(&self) -> &'a str {
        self.source
    }

    pub fn raw_cursor(&self) -> TreeCursor<'a> {
        self.tree.walk()
    }

    pub fn cursor(&'a self) -> Cursor<'a> {
        Cursor::new(self.raw_cursor(), self)
    }
}
