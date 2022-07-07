use crate::cursor::*;
use crate::resolved::*;
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
        let mut cursor = Cursor::new(self.tree.walk(), self);

        // create a closure to give the traverser
        let enter_node = |cur: &Cursor| -> bool {
            match cur.kind() {
                "function_definition" => {
                    if let Some(name) = cur.name() {
                        // add a resolved function
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
            }
            true
        };
    }

    pub fn get_source(&self) -> &'a str {
        self.source
    }
}
