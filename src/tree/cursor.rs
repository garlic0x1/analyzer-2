use super::file::*;
use super::resolved::*;
use super::traverser::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use tree_sitter::*;

pub enum Breaker {
    Continue,
    Break,
    Pass,
}

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

    pub fn iter_all(&self) -> Traversal {
        Traversal::new(self.clone())
    }

    pub fn iter_block(&self) -> Traversal {
        Traversal::new_block(
            self.clone(),
            vec!["method_declaration", "function_definition"],
        )
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

    pub fn goto_next_sibling(&mut self) -> bool {
        self.cursor.goto_next_sibling()
    }

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

        let mut name = String::new();

        for motion in self.iter_all() {
            if let Order::Enter(cur) = motion {
                if cur.kind() == "name" {
                    name = cur.to_string();
                    break;
                }
            }
        }

        Some(name)
    }

    /// resolve all namespaces underneath the cursor
    pub fn resolve(&self) -> HashMap<String, Resolved<'a>> {
        let mut list: HashMap<String, Resolved> = HashMap::new();

        if !self.cursor.clone().goto_parent() {
            list.insert("ROOT".to_owned(), Resolved::new_root(self.clone()));
        }

        // create a closure to give the traverser
        let mut enter_node = |cur: Self, entering: bool| -> Breaker {
            if entering {
                match cur.kind() {
                    "function_definition" => {
                        if let Some(name) = cur.name() {
                            // add a resolved function
                            list.insert(name, Resolved::new_function(cur));
                        }
                    }
                    "method_declaration" => {
                        if let Some(name) = cur.name() {
                            // add a resolved function
                            list.insert(name, Resolved::new_function(cur));
                        }
                    }
                    "class_definition" => {
                        if let Some(name) = cur.name() {
                            // add a resolved function
                        }
                    }
                    "property_name" => {
                        if let Some(name) = cur.name() {
                            // add a resolved function
                        }
                    }
                    _ => (),
                }
            }
            Breaker::Continue
        };

        let mut cur = self.clone();
        cur.traverse(&mut enter_node);

        list
    }
    /// traces up the tree calling closure
    pub fn trace(&mut self, closure: &mut dyn FnMut(Self) -> bool) {
        while self.goto_parent() {
            if self.cursor.node().is_named() {
                if !closure(self.clone()) {
                    break;
                }
            }
        }
    }

    /// accepts a mutable closure to execute on node entry
    pub fn traverse(&mut self, handler: &mut dyn FnMut(Self, bool) -> Breaker) {
        let start_node = self.cursor.node().id();
        let mut visited = false;
        loop {
            let cur = self.clone();
            if visited {
                if self.cursor.goto_next_sibling() {
                    visited = false;
                    if self.cursor.node().is_named() {
                        match handler(self.clone(), true) {
                            Breaker::Continue => continue,
                            Breaker::Break => break,
                            Breaker::Pass => {
                                visited = true;
                                continue;
                            }
                        }
                    }
                } else if self.cursor.goto_parent() {
                    if cur.raw_cursor().node().is_named() {
                        handler(cur, false);
                    }
                    if self.cursor.node().id() == start_node {
                        break;
                    }
                } else {
                    break;
                }
            } else if self.cursor.goto_first_child() {
                if self.cursor.node().is_named() {
                    match handler(self.clone(), true) {
                        Breaker::Continue => continue,
                        Breaker::Break => break,
                        Breaker::Pass => {
                            visited = true;
                            continue;
                        }
                    }
                }
            } else {
                visited = true;
            }
        }
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

    /// get the smallest named node within the current node
    pub fn to_smallest_string(&self) -> Option<String> {
        let node = self.cursor.node();
        let node = node.named_descendant_for_byte_range(node.start_byte(), node.start_byte());

        if let Some(n) = node {
            let slice = &self.file.get_source()[n.byte_range()];
            return Some(slice.to_string());
        }
        None
    }
}
