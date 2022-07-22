use super::resolved::*;
use super::traverser::*;
use crate::tree::cursor::Cursor;
use std::collections::HashMap;
use std::error::Error;
use tree_sitter::*;

pub struct File {
    name: String,
    source: String,
    tree: Tree,
}

impl File {
    pub fn new(name: &str) -> Self {
        let source = std::fs::read_to_string(name).expect(&format!("no such file {}", name));
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_php::language())
            .expect("Error loading PHP parsing support");
        let tree = parser.parse(&source, None).unwrap();
        File::from_tree(name, tree, source)
    }

    pub fn from_url(url: &str) -> Result<Self, Box<dyn Error>> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("g4r1cI's super sweet scanner")
            .build()?;
        let source = client.get(url).send()?.text()?;
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_php::language())
            .expect("Error loading PHP parsing support");
        let tree = parser.parse(&source, None).unwrap();
        Ok(File::from_tree(url, tree, source.clone()))
    }

    pub fn from_tree(name: &str, tree: Tree, source: String) -> Self {
        Self {
            name: name.to_string(),
            source,
            tree,
        }
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

    pub fn iter(&self) -> Traversal {
        Traversal::from_file(self)
    }
}
