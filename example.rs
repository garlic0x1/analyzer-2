use std::collections::HashMap;
use tree_sitter::*;

pub struct File<'a> {
    name: String,
    source: &'a str,
    tree: &'a Tree,
    resolved: HashMap<String, Resolved<'a>>,
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

    pub fn resolve(&mut self) {
        let c = self.tree.walk();
        let mut cursor: Cursor = Cursor::new(c, self);
        let mut list = HashMap::new();

        // create a closure to give the traverser
        let mut enter_node = |cur: &mut Cursor| -> bool {
            match cur.kind() {
                "function_definition" => {
                    if let Some(name) = cur.name() {
                        // add a resolved function
                        let c = cur.clone();
                        list.insert(name.clone(), Resolved::new_function(name, c));
                    }
                }
                _ => (),
            }
            true
        };

        cursor.traverse(&mut enter_node, &mut |_| ());
    }
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
    /// accepts a mutable closure to execute on node entry
    pub fn traverse(
        &mut self,
        enter_node: &mut dyn FnMut(&mut Self) -> bool,
        leave_node: &mut dyn FnMut(&mut Self),
    ) {
        let start_node = self.cursor.node().id();
        let mut visited = false;
        loop {
            if visited {
                if self.cursor.goto_next_sibling() {
                    if self.cursor.node().is_named() {
                        if enter_node(self) {
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
                    if self.cursor.node().is_named() {
                        leave_node(&mut self);
                    }
                    if self.cursor.node().id() == start_node {
                        break;
                    }
                } else {
                    break;
                }
            } else if self.cursor.goto_first_child() {
                if self.cursor.node().is_named() {
                    if enter_node(&mut self) {
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
}
