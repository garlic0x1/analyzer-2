use super::file::*;
use super::resolved::*;
use std::collections::HashMap;
use tree_sitter::*;

pub enum Breaker {
    Continue,
    Break,
    Pass,
}

#[derive(Clone)]
pub struct Cursor<'a> {
    cursor: TreeCursor<'a>,
    file: &'a File<'a>,
}

impl<'a> Cursor<'a> {
    pub fn new(cursor: TreeCursor<'a>, file: &'a File<'a>) -> Self {
        Self { cursor, file }
    }

    pub fn from_file(file: &'a File<'a>) -> Self {
        Self {
            cursor: file.raw_cursor(),
            file,
        }
    }

    pub fn kind(&self) -> &str {
        self.cursor.node().kind()
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
        let mut name = String::new();

        // create a mutable closure, and capture the string to mutate
        let mut enter_node = |cur: Self| -> Breaker {
            if cur.cursor.node().kind() == "name" {
                // return false to stop crawling
                name = cur.to_string();
                Breaker::Break
            } else {
                // continue crawling
                Breaker::Continue
            }
        };

        // traverse a clone for self immutability
        let mut cur = self.clone();
        cur.traverse(&mut enter_node, &mut |_| ());

        Some(name)
    }

    /// resolve all namespaces underneath the cursor
    pub fn resolve(&self) -> HashMap<String, Resolved<'a>> {
        let mut list: HashMap<String, Resolved> = HashMap::new();

        if !self.cursor.clone().goto_parent() {
            list.insert("ROOT".to_owned(), Resolved::new_root(self.clone()));
        }

        // create a closure to give the traverser
        let mut enter_node = |cur: Self| -> Breaker {
            match cur.kind() {
                "function_definition" => {
                    if let Some(name) = cur.name() {
                        // add a resolved function
                        list.insert(name, Resolved::new_function(cur));
                    }
                }
                "method_definition" => {
                    if let Some(name) = cur.name() {
                        // add a resolved function
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
            Breaker::Continue
        };

        let mut cur = self.clone();
        cur.traverse(&mut enter_node, &mut |_| ());

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
    pub fn traverse(
        &mut self,
        enter_node: &mut dyn FnMut(Self) -> Breaker,
        leave_node: &mut dyn FnMut(Self),
    ) {
        let start_node = self.cursor.node().id();
        let mut visited = false;
        loop {
            if visited {
                if self.cursor.goto_next_sibling() {
                    visited = false;
                    if self.cursor.node().is_named() {
                        match enter_node(self.clone()) {
                            Breaker::Continue => continue,
                            Breaker::Break => break,
                            Breaker::Pass => {
                                if self.cursor.goto_next_sibling() {
                                    continue;
                                } else if self.cursor.goto_parent() {
                                    if self.cursor.node().is_named() {
                                        leave_node(self.clone());
                                    }
                                    if self.cursor.node().id() == start_node {
                                        break;
                                    }
                                    //visited = true;
                                    continue;
                                }
                            }
                        }
                    }
                } else if self.cursor.goto_parent() {
                    if self.cursor.node().is_named() {
                        leave_node(self.clone());
                    }
                    if self.cursor.node().id() == start_node {
                        break;
                    }
                } else {
                    break;
                }
            } else if self.cursor.goto_first_child() {
                if self.cursor.node().is_named() {
                    match enter_node(self.clone()) {
                        Breaker::Continue => continue,
                        Breaker::Break => break,
                        Breaker::Pass => {
                            if self.cursor.goto_next_sibling() {
                                continue;
                            } else if self.cursor.goto_parent() {
                                if self.cursor.node().is_named() {
                                    leave_node(self.clone());
                                }
                                if self.cursor.node().id() == start_node {
                                    break;
                                }
                                //visited = true;
                                continue;
                            }
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
