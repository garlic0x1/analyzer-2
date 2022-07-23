use super::file::*;
use super::tracer::Trace;
use super::traverser::*;
use std::hash::{Hash, Hasher};
use tree_sitter::*;

#[derive(Clone)]
pub struct Cursor<'a> {
    cursor: TreeCursor<'a>,
    file: &'a File,
}

impl<'a> Eq for Cursor<'a> {}

impl<'a> PartialEq for Cursor<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.file.name() == other.file.name() && self.cursor.node().id() == other.cursor.node().id()
    }
}

impl<'a> Hash for Cursor<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.file.name().hash(state);
        self.cursor.node().id().hash(state);
    }
}

impl<'a> std::fmt::Debug for Cursor<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = self.kind().to_string();
        if let Some(name) = self.name() {
            s.push_str(&name);
        }

        write!(f, "{}", s)
    }
}

impl<'a> Cursor<'a> {
    pub fn new(cursor: TreeCursor<'a>, file: &'a File) -> Self {
        Self { cursor, file }
    }

    pub fn from_file(file: &'a File) -> Self {
        Self {
            cursor: file.raw_cursor(),
            file,
        }
    }

    pub fn filename(&self) -> String {
        self.file.name()
    }

    pub fn kind(&self) -> &str {
        self.cursor.node().kind()
    }

    pub fn field(&self) -> Option<&str> {
        self.cursor.field_name()
    }

    pub fn traverse(&self) -> Traversal {
        Traversal::new(&self)
    }

    pub fn traverse_block(&self) -> Traversal {
        Traversal::new_block(&self, vec!["method_declaration", "function_definition"])
    }

    pub fn trace(&self) -> Trace {
        Trace::new(self.clone())
    }

    pub fn raw_cursor(&self) -> TreeCursor<'a> {
        self.cursor.clone()
    }

    pub fn goto_parent(&mut self) -> bool {
        self.cursor.goto_parent()
    }

    pub fn goto_first_child(&mut self) -> bool {
        self.cursor.goto_first_child()
    }

    pub fn goto_child(&mut self, index: usize) -> bool {
        if !self.goto_first_child() {
            return false;
        }
        let mut i = 0;
        while i < index {
            if !self.goto_next_sibling() {
                return false;
            }
            i += 1;
        }
        true
    }

    pub fn goto_field(&mut self, field: &str) -> bool {
        self.goto_first_child();
        while self.field() != Some(field) {
            if !self.goto_next_sibling() {
                self.goto_parent();
                return false;
            }
        }
        true
    }

    pub fn goto_next_sibling(&mut self) -> bool {
        self.cursor.goto_next_sibling()
    }

    /// try to find the name of current node
    pub fn name(&self) -> Option<String> {
        // handle name nodes properly
        if self.kind() == "name" {
            return Some(self.to_string());
        }

        // hacky fix because depth first incorrect for method calls
        {
            let mut cur = self.clone();
            cur.goto_first_child();
            while cur.goto_next_sibling() {
                if cur.kind() == "name" {
                    return Some(cur.to_string());
                }
            }
        }

        for motion in self.traverse() {
            if let Order::Enter(cur) = motion {
                if cur.kind() == "name" {
                    return Some(cur.to_string());
                }
            }
        }

        None
    }

    /// get which child index we are in
    pub fn get_index(&self) -> usize {
        let node_id = self.cursor.node().id();
        let mut index = 0;
        let mut cursor = self.cursor.clone();

        // handle cases where we are checking root node
        if !cursor.goto_parent() {
            return 0;
        }

        cursor.goto_first_child();
        while cursor.node().id() != node_id {
            if cursor.node().is_named() {
                index += 1;
            }
            cursor.goto_next_sibling();
        }

        index
    }

    /// get the source code of the current node
    pub fn to_string(&self) -> String {
        let node = self.cursor.node();
        let slice = &self.file.get_source()[node.byte_range()];
        slice.to_string()
    }

    /// get the source code of the current node
    pub fn to_str(&self) -> &str {
        let node = self.cursor.node();
        let slice = &self.file.get_source()[node.byte_range()];
        slice
    }
}
