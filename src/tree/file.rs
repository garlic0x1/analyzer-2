use super::cursor::Cursor;
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
