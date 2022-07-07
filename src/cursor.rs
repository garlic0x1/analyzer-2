use crate::node_to_string;
use crate::resolver::File;
use tree_sitter::*;

#[derive(Clone)]
pub struct Cursor<'a> {
    cursor: TreeCursor<'a>,
    file: &'a File<'a>,
}

impl<'a> Cursor<'a> {
    pub fn new(cursor: TreeCursor<'a>, file: &'a File<'a>) -> Self {
        Self { cursor, file }
    }

    pub fn kind(&self) -> &str {
        self.cursor.node().kind()
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
            println!("{}", i);
        }
        true
    }

    pub fn goto_next_sibling(&mut self) -> bool {
        self.cursor.goto_next_sibling()
    }

    pub fn name(&mut self) -> Option<String> {
        let mut name = String::new();

        // create a mutable closure, and capture the string to mutate
        let mut enter_node = |cur: &Self| -> bool {
            if cur.cursor.node().kind() == "name" {
                name = cur.to_string();
                // return false to stop crawling
                false
            } else {
                // continue crawling
                true
            }
        };

        self.traverse(&mut enter_node, &mut |_| ());
        return Some(name);
    }

    /// accepts a mutable closure to execute on node entry
    pub fn traverse(
        &mut self,
        enter_node: &mut dyn FnMut(&Self) -> bool,
        leave_node: &mut dyn FnMut(&Self),
    ) {
        let start_node = self.cursor.node().id();
        let mut visited = false;
        loop {
            if visited {
                if self.cursor.goto_next_sibling() {
                    if self.cursor.node().is_named() {
                        if enter_node(&self) {
                            continue;
                        }
                        if self.cursor.goto_next_sibling() {
                            visited = false;
                            continue;
                        } else if self.cursor.goto_parent() {
                            visited = true;
                            continue;
                        }
                    }
                    visited = false;
                } else if self.cursor.goto_parent() {
                    if self.cursor.node().id() == start_node {
                        break;
                    }
                } else {
                    break;
                }
            } else if self.cursor.goto_first_child() {
                if self.cursor.node().is_named() {
                    if enter_node(&self) {
                        continue;
                    }
                    if self.cursor.goto_next_sibling() {
                        visited = false;
                        continue;
                    } else if self.cursor.goto_parent() {
                        visited = true;
                        continue;
                    }
                }
            } else {
                visited = true;
            }
        }
    }

    pub fn get_index(&self) -> usize {
        let node_id = self.cursor.node().id();
        let mut index = 0;
        let mut cursor = self.cursor.clone();
        cursor.goto_parent();
        cursor.goto_first_child();
        while cursor.node().id() != node_id {
            if cursor.node().is_named() {
                index += 1;
            }
            cursor.goto_next_sibling();
        }

        index
    }

    pub fn to_string(&self) -> String {
        let node = self.cursor.node();
        let slice = &self.file.source_code[node.byte_range()];
        slice.to_string()
    }
}
