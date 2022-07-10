use crate::analyzer::taint::*;
use crate::tree::cursor::*;
use crate::tree::file::*;
use crate::tree::resolved::*;

pub struct Analyzer<'a> {
    taints: TaintList,
    context: ContextStack,
    files: Vec<&'a File<'a>>,
}

impl<'a> Analyzer<'a> {
    pub fn new(files: Vec<&'a File<'a>>) -> Self {
        Self {
            taints: TaintList::new(),
            context: ContextStack::new(),
            files,
        }
    }

    /// begins analysis assumming the first file is the main/starting file
    pub fn analyze(&mut self) {
        self.traverse(Cursor::from_file(
            self.files.get(0).expect("no files provided"),
        ));
    }

    /// traverse the program, looking for taints to trace, and following program flow
    fn traverse(&mut self, cursor: Cursor) {
        let mut closure = |cur: Cursor| -> Breaker {
            match cur.kind() {
                "variable_name" => {
                    // check for taint and trace
                    self.trace(cur);
                    Breaker::Continue
                }
                // do not crawl into these node types
                "function_definition" => Breaker::Pass,
                _ => Breaker::Continue,
            }
        };

        let mut cursor = cursor.clone();
        cursor.traverse(&mut closure, &mut |_| ());
    }

    fn trace(&mut self, cursor: Cursor) {
        let mut path = Vec::new();
        let mut closure = |cur: Cursor| -> bool {
            match cur.kind() {
                "expression_statement" => false,
                "function_call_expression" => {
                    //path.push(cur.name());
                    //println!("{:?}", cur.name());
                    path.push(cur.name());
                    true
                }
                _ => true,
            }
        };
        let mut cursor = cursor;
        println!("{}, {:?}", cursor.kind(), cursor.name());
        cursor.trace(&mut closure);
        println!("{:?}", path);
    }
}
