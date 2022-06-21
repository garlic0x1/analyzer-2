use tree_sitter::*;
use std::collections::HashMap;

#[derive(Clone)]
pub enum Resolved<'a> {
    Function { name: String, cursor: TreeCursor<'a> },
    Class { name: String, cursor: TreeCursor<'a> },
    Method { name: String, cursor: TreeCursor<'a> },
    Property { name: String, cursor: TreeCursor<'a> },
}

#[derive(Clone)]
pub struct File<'a> {
    pub filename: String,
    pub source_code: &'a str,
    pub tree: &'a Tree,
    pub resolved: HashMap<String, Resolved<'a>>,
}

// file should contain all traversing logic?
impl<'a> File<'a> {
    pub fn new(filename: String, tree: &'a Tree, source_code: &'a str) -> Self {
        Self {
            filename,
            source_code,
            tree: &tree,
            resolved: HashMap::new(),
        }
    }

    pub fn is_main(&self) -> bool {
        self.source_code.contains("* Plugin Name: ")
    }

    // crawl tree and identify code blocks
    pub fn resolve(&mut self) {
        let mut cursor = self.tree.walk();
        let mut visited = false;
        loop {
            if visited {
                if cursor.goto_next_sibling() {
                    // enter
                    self.resolve_node(&cursor.clone());
                } else if cursor.goto_parent() {
                } else {
                    break;
                }
            } else if cursor.goto_first_child() {
                // enter
                self.resolve_node(&cursor.clone());
            } else {
                visited = true;
            }
        }
        println!("Resolved functions {:?}", self.resolved.clone().into_keys());
    }

    fn resolve_node(&mut self, cursor: &TreeCursor<'a>) {
        let node = cursor.node();
        match node.kind() {
            "function_definition" => {
                if let Ok(name) = self.find_name(&mut cursor.clone()) {
                let value = Resolved::Function {
                    name: name.clone(),
                    cursor: cursor.clone(),
                };
                self.resolved.insert(name, value);
                }
            }
            "method_definition" => {
                if let Ok(name) = self.find_name(&mut cursor.clone()) {
                let value = Resolved::Method {
                    name: name.clone(),
                    cursor: cursor.clone(),
                };
                self.resolved.insert(name, value);
                }
            }
            "property_name" => {
                if let Ok(name) = self.find_name(&mut cursor.clone()) {
                let value = Resolved::Property {
                    name: name.clone(),
                    cursor: cursor.clone(),
                };
                self.resolved.insert(name, value);
                }
            }
            "class_definition" => {
                if let Ok(name) = self.find_name(&mut cursor.clone()) {
                let value = Resolved::Class {
                    name: name.clone(),
                    cursor: cursor.clone(),
                };
                self.resolved.insert(name, value);
                }
            }
            _ => (),
        }
    }

    fn find_name(&self, cursor: &mut TreeCursor) -> Result<String, ()> {
        let mut visited = false;
        loop {
            if visited {
                if cursor.goto_next_sibling() {
                    visited = false;
                    if cursor.node().kind() == "name" {
                        let s: String = node_to_string(&cursor.node(), self.source_code);
                        return Ok(s);
                    }
                } else if cursor.goto_parent() {
                } else {
                    break;
                }
            } else if cursor.goto_first_child() {
                if cursor.node().kind() == "name" {
                    let s: String = node_to_string(&cursor.node(), self.source_code);
                    return Ok(s);
                }
            } else {
                visited = true;
            }
        }

        Err(())
    }
}

fn node_to_string(node: &Node, source: &str) -> String {
    let slice = &source[node.start_byte()..node.end_byte()];
    slice.to_string()
}
