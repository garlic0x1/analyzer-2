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

    pub fn resolve(&'a mut self) {
        let c = self.tree.walk();
        let mut cursor = Cursor::new(c, self);

        // create a closure to give the traverser
        let mut enter_node = |cur: &Cursor| -> bool {
            match cur.kind() {
                "function_definition" => {
                    if let Some(name) = cur.name() {
                        // add a resolved function
                        self.resolved
                            .insert(name, Resolved::new_function(name, cursor.clone()));
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
            true
        };

        cursor.traverse(&mut enter_node, &mut |_| ());
    }

    pub fn get_source(&self) -> &'a str {
        self.source
    }
}
