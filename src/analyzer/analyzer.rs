use crate::analyzer::taint::*;
use crate::tree::cursor::*;
use crate::tree::file::*;
use crate::tree::resolved::*;
use tree_sitter::*;

pub struct Analyzer<'a> {
    taints: Vec<Taint>,
    context_stack: Vec<Context>,
    files: &'a Vec<File<'a>>,
}

impl<'a> Analyzer<'a> {
    pub fn new(files: &'a Vec<File<'a>>) -> Self {
        Self {
            taints: Vec::new(),
            context_stack: Vec::new(),
            files,
        }
    }

    pub fn traverse(&mut self, cursor: &mut Cursor) {
        let mut closure = |cur: Cursor| -> bool {
            match cur.kind() {
                "variable_name" => {
                    // check for taint
                    true
                }
                "function_definition" => false,
                _ => true,
            }
        };

        cursor.traverse(&mut closure, &mut |_| ());
    }
}
