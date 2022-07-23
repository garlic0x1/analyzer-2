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

    /// associated function to dump individual cursors
    pub fn dump_cursor(cursor: Cursor<'a>) -> String {
        let mut string = String::new();
        let mut depth = 0;

        for cur in cursor.traverse() {
            match cur {
                Order::Enter(cur) => {
                    let indent = "  ".repeat(depth);
                    string.push_str(&format!("{}Kind: {}\n", indent, cur.kind()));
                    if cur.kind() == "name" {
                        string.push_str(&format!("{}Name: {}\n", indent, cur.to_str()));
                    }
                    if let Some(field) = cur.raw_cursor().field_name() {
                        string.push_str(&format!("{}Field: {}\n", indent, field));
                    }

                    depth += 1;
                }
                Order::Leave(_) => {
                    depth -= 1;
                }
            }
        }

        string
    }

    pub fn dump(&self) -> String {
        let mut string = String::new();

        for file in self.files.iter() {
            string.push_str(&Dumper::dump_cursor(file.cursor()));
        }

        string
    }
}
