use crate::analyzer::taint::*;
use crate::tree::cursor::*;
use crate::tree::file::*;
use crate::tree::resolved::*;
use tree_sitter::*;

pub struct Analyzer<'a> {
    taints: Vec<Taint<'a>>,
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
}
