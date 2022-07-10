use crate::analyzer::taint::*;
use crate::tree::cursor::*;
use crate::tree::file::*;
use crate::tree::resolved;
use crate::tree::resolved::*;
use std::collections::HashMap;

pub struct Analyzer<'a> {
    taints: TaintList,
    context: ContextStack,
    files: Vec<&'a File<'a>>,
    resolved: HashMap<String, Resolved<'a>>,
}

impl<'a> Analyzer<'a> {
    pub fn new(files: Vec<&'a File<'a>>) -> Self {
        Self {
            taints: TaintList::new(),
            context: ContextStack::new(),
            files,
            resolved: HashMap::new(),
        }
    }

    /// returns a new analyzer with sources to trace
    pub fn from_sources(files: Vec<&'a File<'a>>, sources: Vec<String>) -> Self {
        let mut taints = TaintList::new();
        for source in sources {
            taints.push(Taint::new_global(source));
        }
        Self {
            files,
            taints,
            context: ContextStack::new(),
            resolved: HashMap::new(),
        }
    }

    /// first resolves names
    /// then begins traversal
    pub fn analyze(&mut self) {
        for file in self.files.iter() {
            let cur = Cursor::from_file(file);
            self.resolved.extend(cur.resolve());
        }

        self.traverse(Cursor::from_file(
            self.files.get(0).expect("no files provided"),
        ));
    }

    /// traverse the program, looking for taints to trace, and following program flow
    fn traverse(&mut self, cursor: Cursor) {
        let mut closure = |cur: Cursor| -> Breaker {
            match cur.kind() {
                "variable_name" => {
                    // check if in left of assignment and return
                    if let Some(s) = cur.raw_cursor().field_name() {
                        if s == "left" {
                            return Breaker::Continue;
                        }
                    }

                    // check for taint and trace
                    if self.taints.contains(&Taint::new_variable(cur.clone())) {
                        self.trace(cur);
                    }
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

    /// trace taints up the tree
    fn trace(&mut self, cursor: Cursor) {
        let mut path = Vec::new();
        let mut closure = |cur: Cursor| -> bool {
            match cur.kind() {
                "expression_statement" => false,
                "assignment_expression" => {
                    self.taints.push(Taint::new_variable(cur.clone()));
                    path.push(format!("assign {}", cur.name().unwrap()));
                    false
                }
                // todo: pass param taint
                "function_call_expression" => {
                    if let Some(resolved) = self.resolved.clone().get(&cur.name().unwrap()) {
                        self.context.push(Context::new(
                            resolved.cursor().kind().to_string(),
                            resolved.cursor().name().unwrap(),
                        ));
                        self.traverse(resolved.cursor());
                        self.context.pop();
                    }
                    path.push(cur.name().unwrap());
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
