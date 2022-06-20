use tree_sitter::*;

#[derive(Debug, Clone)]
pub enum Resolved {
    Function { name: String, point: Point },
    Class { name: String, point: Point },
    Method { name: String, point: Point },
    Property { name: String, point: Point },
}

#[derive(Debug, Clone)]
pub struct File {
    pub filename: String,
    pub source_code: String,
    pub tree: Tree,
    pub resolved: Vec<Resolved>,
}

impl File {
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
            resolved: Vec::new(),
        }
    }

    pub fn is_main(&self) -> bool {
        self.source_code.contains("* Plugin Name: ")
    }

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
