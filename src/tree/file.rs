use super::cursor::Cursor;
use tree_sitter::*;

pub struct File<'a> {
    name: String,
    source: &'a str,
    tree: Tree,
}

impl<'a> File<'a> {
    pub fn new(name: String, source: &'a str) -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_php::language())
            .expect("Error loading PHP parsing support");
        let tree = parser.parse(&source, None).unwrap();
        File::from_tree(name, tree, source)
    }

    pub fn from_tree(name: String, tree: Tree, source: &'a str) -> Self {
        Self { name, source, tree }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn get_source(&self) -> &'a str {
        self.source
    }

    pub fn raw_cursor(&self) -> TreeCursor {
        self.tree.walk()
    }

    pub fn cursor(&'a self) -> Cursor<'a> {
        Cursor::new(self.raw_cursor(), self)
    }
}
