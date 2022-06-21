use tree_sitter::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Resolved<'a> {
    Function { name: String, point: Point, cursor: TreeCursor<'a> },
    Class { name: String, point: Point, cursor: TreeCursor<'a> },
    Method { name: String, point: Point, cursor: TreeCursor<'a> },
    Property { name: String, point: Point, cursor: TreeCursor<'a> },
}

#[derive(Debug, Clone)]
pub struct File<'a> {
    pub filename: String,
    pub source_code: String,
    pub tree: Tree,
    pub resolved: HashMap<String, Resolved<'a>>,
}

// file should contain all traversing logic?
impl<'a> File<'a> {
    pub fn new(filename: String, source_code: String) -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_php::language())
            .expect("Error loading PHP parsing support");
        let tree = parser.parse(source_code.clone(), None).unwrap();
        Self {
            filename,
            source_code,
            tree,
            resolved: HashMap::new(),
        }
    }

    pub fn is_main(&self) -> bool {
        self.source_code.contains("* Plugin Name: ")
    }

    // crawl tree and identify code blocks
    fn resolve(&mut self) {
        let t = self.tree.clone();
        let mut cursor = t.walk();
        let mut visited = false;
        loop {
            if visited {
                if cursor.goto_next_sibling() {
                    // enter
                } else if cursor.goto_parent() {
                } else {
                    break;
                }
            } else if cursor.goto_first_child() {
                // enter
            } else {
                visited = true;
            }
        }
    }

    fn enter_node(&mut self, cursor: &mut TreeCursor) {
        let node = cursor.node();
        match node.kind() {
            "function_definition" => {}
            "method_definition" => {}
            "property_name" => {}
            "class_definition" => {}
            _ => (),
        }
    }

    fn find_name(&self, cursor: &mut TreeCursor, file: &File) -> Result<String, ()> {
        let mut visited = false;
        loop {
            if visited {
                if cursor.goto_next_sibling() {
                    visited = false;
                    if cursor.node().kind() == "name" {
                        let s: String = node_to_string(&cursor.node(), file.source_code.as_str());
                        return Ok(s);
                    }
                } else if cursor.goto_parent() {
                } else {
                    break;
                }
            } else if cursor.goto_first_child() {
                if cursor.node().kind() == "name" {
                    let s: String = node_to_string(&cursor.node(), file.source_code.as_str());
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
