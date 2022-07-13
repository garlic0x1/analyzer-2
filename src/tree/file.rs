use super::cursor::Cursor;
use std::error::Error;
use tree_sitter::*;

pub struct File {
    name: String,
    source: String,
    tree: Tree,
}

impl File {
    pub fn new(name: &str, source: String) -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_php::language())
            .expect("Error loading PHP parsing support");
        let tree = parser.parse(&source, None).unwrap();
        File::from_tree(name, tree, source)
    }

    pub fn from_url(url: &str) -> Result<Self, Box<dyn Error>> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("plugin scanner")
            .build()?;
        let body = client.get(url).send()?.text()?;
        println!("{}", body);

        let source = body;
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
}
