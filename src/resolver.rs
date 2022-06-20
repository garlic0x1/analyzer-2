use tree_sitter::*;

#[derive(Debug, Clone)]
pub enum Resolved {
    Function { name: String },
    Class { name: String },
    Method { name: String },
    Property { name: String },
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

    pub fn is_main() -> bool {
        true
    }
}
