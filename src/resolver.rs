use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use tree_sitter::*;

#[derive(Clone)]
pub enum Resolved<'a> {
    Root {
        cursor: TreeCursor<'a>,
    },
    Function {
        name: String,
        cursor: TreeCursor<'a>,
        params: Vec<String>,
    },
    Class {
        name: String,
        cursor: TreeCursor<'a>,
    },
    Method {
        name: String,
        cursor: TreeCursor<'a>,
        params: Vec<String>,
    },
    Property {
        name: String,
        cursor: TreeCursor<'a>,
    },
}

#[derive(Clone)]
pub struct File<'a> {
    pub filename: String,
    pub source_code: &'a str,
    pub tree: &'a Tree,
    pub resolved: HashMap<String, Resolved<'a>>,
}
impl<'a> fmt::Debug for File<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Resolved").field("filename", &self).finish()
    }
}

impl<'a> Hash for File<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.filename.hash(state);
    }
}

impl<'a> PartialEq for File<'a> {
    fn eq(&self, other: &Self) -> bool {
        return self.filename == other.filename;
    }
}

impl<'a> File<'a> {
    pub fn new(filename: String, tree: &'a Tree, source_code: &'a str) -> Self {
        let mut s = Self {
            filename,
            source_code,
            tree: &tree,
            resolved: HashMap::new(),
        };
        s.resolve();
        return s;
    }

    pub fn is_main(&self) -> bool {
        self.source_code.contains("* Plugin Name: ")
    }

    // crawl tree and identify code blocks
    fn resolve(&mut self) {
        let mut cursor = self.tree.walk();

        let resolved = Resolved::Root {
            cursor: cursor.clone(),
        };
        self.resolved.insert("ROOT".to_string(), resolved);

        let start_node = cursor.node().id();
        let mut visited = false;
        loop {
            //println!("name: {:?}", cursor.node().kind());
            if visited {
                if cursor.goto_next_sibling() {
                    // enter
                    self.resolve_node(&cursor.clone());
                } else if cursor.goto_parent() {
                    if cursor.node().id() == start_node {
                        break;
                    }
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
    }

    fn get_params(&self, cursor: &mut TreeCursor) -> Vec<String> {
        println!(
            "getting params: {}",
            node_to_string(&cursor.node(), self.source_code)
        );
        let mut params: Vec<String> = Vec::new();
        let start_node = cursor.node().id();
        let mut visited = false;
        loop {
            if visited {
                if cursor.goto_next_sibling() {
                    // enter
                    if cursor.node().kind() == "simple_parameter" {
                        println!(
                            "parameter: {}",
                            node_to_string(&cursor.node(), self.source_code)
                        );
                        for name in self.find_name(&mut cursor.clone()) {
                            params.push(name);
                        }
                    }
                    visited = false;
                } else if cursor.goto_parent() {
                    if cursor.node().id() == start_node {
                        break;
                    }
                } else {
                    break;
                }
            } else if cursor.goto_first_child() {
                // enter
                if cursor.node().kind() == "simple_parameter" {
                    println!(
                        "parameter: {}",
                        node_to_string(&cursor.node(), self.source_code)
                    );
                    for name in self.find_name(&mut cursor.clone()) {
                        params.push(name);
                    }
                }
            } else {
                visited = true;
            }
        }

        println!("params: {:?}", params);
        params
    }

    fn resolve_node(&mut self, cursor: &TreeCursor<'a>) {
        //println!("name: {:?}", cursor.node().kind());
        let node = cursor.node();
        match node.kind() {
            "function_definition" => {
                for name in self.find_name(&mut cursor.clone()) {
                    let value = Resolved::Function {
                        name: name.clone(),
                        cursor: cursor.clone(),
                        params: self.get_params(&mut cursor.clone()),
                    };
                    self.resolved.insert(name, value);
                }
            }
            "method_definition" => {
                for name in self.find_name(&mut cursor.clone()) {
                    let value = Resolved::Method {
                        name: name.clone(),
                        cursor: cursor.clone(),
                        params: self.get_params(&mut cursor.clone()),
                    };
                    self.resolved.insert(name, value);
                }
            }
            "property_name" => {
                for name in self.find_name(&mut cursor.clone()) {
                    let value = Resolved::Property {
                        name: name.clone(),
                        cursor: cursor.clone(),
                    };
                    self.resolved.insert(name, value);
                }
            }
            "class_definition" => {
                for name in self.find_name(&mut cursor.clone()) {
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

    pub fn find_name(&self, cursor: &mut TreeCursor) -> Option<String> {
        let mut visited = false;
        let start_node = cursor.node().id();
        loop {
            if visited {
                if cursor.goto_next_sibling() {
                    visited = false;
                    if cursor.node().kind() == "name" {
                        let s: String = node_to_string(&cursor.node(), self.source_code);
                        return Some(s);
                    }
                } else if cursor.goto_parent() {
                    if cursor.node().id() == start_node {
                        break;
                    }
                } else {
                    break;
                }
            } else if cursor.goto_first_child() {
                if cursor.node().kind() == "name" {
                    let s: String = node_to_string(&cursor.node(), self.source_code);
                    return Some(s);
                }
            } else {
                visited = true;
            }
        }

        None
    }
}

fn node_to_string(node: &Node, source: &str) -> String {
    let slice = &source[node.start_byte()..node.end_byte()];
    slice.to_string()
}
