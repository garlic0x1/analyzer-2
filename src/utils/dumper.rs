use crate::tree::cursor::*;
use crate::tree::file::*;
use crate::tree::resolved::*;

pub struct Dumper<'a> {
    files: Vec<&'a File>,
}

impl<'a> Dumper<'a> {
    /// create a dumper from a vec of files
    pub fn new(files: Vec<&'a File>) -> Self {
        Self { files }
    }

    /// dump the tree as a string
    pub fn dump(&self) -> String {
        let mut string = String::new();
        let mut node_handler = |cur: Cursor, entering: bool| -> Breaker {
            if entering {
                let indent = "  ".repeat(Dumper::depth(cur.clone()));
                string.push_str(&format!("{}Kind: {}\n", indent, cur.kind()));
                string.push_str(&format!("{}Name: {:?}\n", indent, cur.name()));
            }
            Breaker::Continue
        };

        for file in self.files.iter() {
            file.cursor().traverse(&mut node_handler);
        }

        string
    }

    pub fn resolved(&self) -> String {
        let mut string = String::new();
        for file in self.files.iter() {
            for (_, r) in file.cursor().resolve().iter() {
                if let Resolved::Function { cursor } = r {
                    string.push_str(&format!(
                        "{}: {}",
                        cursor.name().unwrap(),
                        r.dump_parameters()
                    ));
                }
            }
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
