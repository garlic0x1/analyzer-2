use crate::tree::cursor::*;
use crate::tree::file;
use crate::tree::file::*;
use crate::tree::resolved::*;
use tree_sitter::*;

pub struct Dumper<'a> {
    files: Vec<&'a File<'a>>,
}

impl<'a> Dumper<'a> {
    /// create a dumper from a vec of files
    pub fn new(files: Vec<&'a File<'a>>) -> Self {
        Self { files }
    }

    /// dump the tree as a string
    pub fn dump(&self) -> String {
        let mut string = String::new();
        let mut node_handler = |cur: Cursor| -> Breaker {
            let indent = "  ".repeat(Dumper::depth(cur.clone()));
            string.push_str(&format!("{}Kind: {}\n", indent, cur.kind()));
            string.push_str(&format!("{}Name: {:?}\n", indent, cur.name()));
            Breaker::Continue
        };

        for file in self.files.iter() {
            file.cursor().traverse(&mut node_handler, &mut |_| ());
        }

        string
    }

    /// determine the depth of the node
    fn depth(cursor: Cursor) -> usize {
        let mut d = 0;
        let mut tracer = |_: Cursor| -> bool {
            d += 1;
            true
        };

        let mut cursor = cursor;
        cursor.trace(&mut tracer);

        d
    }
}
