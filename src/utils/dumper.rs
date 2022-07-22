use crate::tree::cursor::*;
use crate::tree::file::*;
use crate::tree::resolved::*;
use crate::tree::traverser::*;

pub struct Dumper<'a> {
    files: Vec<&'a File>,
}

impl<'a> Dumper<'a> {
    /// create a dumper from a vec of files
    pub fn new(files: Vec<&'a File>) -> Self {
        Self { files }
    }

    pub fn dump_pass(cursor: Cursor<'a>) -> String {
        let mut string = String::new();

        let mut traversal = Traversal::new(cursor);

        while let Some(cur) = traversal.next() {
            match cur {
                Order::Enter(cur) => {
                    if cur.raw_cursor().node().is_named() {
                        if cur.kind() == "method_declaration" {
                            traversal.pass();
                        }
                        let indent = "  ".repeat(Dumper::depth(cur.clone()));
                        string.push_str(&format!("{}Kind: {} {{\n", indent, cur.kind()));
                    }
                }
                Order::Leave(cur) => {
                    if cur.raw_cursor().node().is_named() {
                        let indent = "  ".repeat(Dumper::depth(cur.clone()));
                        string.push_str(&format!("{}}}\n", indent));
                    }
                }
            }
        }

        string
    }
    pub fn dump_cursor(cursor: Cursor<'a>) -> String {
        let mut string = String::new();

        for cur in cursor.iter() {
            match cur {
                Order::Enter(cur) => {
                    if cur.raw_cursor().node().is_named() {
                        let indent = "  ".repeat(Dumper::depth(cur.clone()));
                        string.push_str(&format!("{}Kind: {} {{\n", indent, cur.kind()));
                    }
                }
                Order::Leave(cur) => {
                    if cur.raw_cursor().node().is_named() {
                        let indent = "  ".repeat(Dumper::depth(cur.clone()));
                        string.push_str(&format!("{}}}\n", indent));
                    }
                }
            }
        }

        string
    }

    pub fn dump(&self) -> String {
        let mut string = String::new();

        for file in self.files.iter() {
            string.push_str(&Dumper::dump_pass(file.cursor()));
        }

        let string2 = self.dump2();
        println!("{}", string == string2);

        string
    }

    /// dump the tree as a string
    pub fn dump2(&self) -> String {
        let mut string = String::new();
        let mut node_handler = |cur: Cursor, entering: bool| -> Breaker {
            if entering {
                let indent = "  ".repeat(Dumper::depth(cur.clone()));
                string.push_str(&format!("{}Kind: {}\n", indent, cur.kind()));
                if let Some(field) = cur.raw_cursor().field_name() {
                    string.push_str(&format!("{}Field: {}\n", indent, field));
                }
                if cur.kind() == "name" {
                    string.push_str(&format!("{}Name: {:?}\n", indent, cur.to_string()));
                }
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
