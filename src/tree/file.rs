use super::cursor::Cursor;
use std::error::Error;
use tree_sitter::*;

pub struct File {
    name: String,
    source: String,
    tree: Tree,
}

impl File {
    pub fn new(name: String, source: String) -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_php::language())
            .expect("Error loading PHP parsing support");
        let tree = parser.parse(&source, None).unwrap();
        File::from_tree(name, tree, source)
    }

    pub fn from_url(url: String) -> Result<Self, Box<dyn Error>> {
        let body = reqwest::blocking::get("https://www.rust-lang.org")?.text()?;

        let source = body;
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_php::language())
            .expect("Error loading PHP parsing support");
        let tree = parser.parse(&source, None).unwrap();
        Ok(File::from_tree(url, tree, source.clone()))
    }

    pub fn from_tree(name: String, tree: Tree, source: String) -> Self {
        Self { name, source, tree }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn get_source(&self) -> &str {
        &self.source
    }

    pub fn raw_cursor(&self) -> TreeCursor {
        self.tree.walk()
    }

    pub fn cursor(&self) -> Cursor {
        Cursor::new(self.raw_cursor(), self)
    }
}
