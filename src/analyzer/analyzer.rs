use crate::analyzer::taint::*;
use crate::graph::graph::*;
use crate::tree::cursor::*;
use crate::tree::file::*;
use crate::tree::resolved::*;
use std::collections::HashMap;

pub struct Analyzer<'a> {
    taints: TaintList,
    context: ContextStack,
    files: Vec<&'a File<'a>>,
    resolved: HashMap<String, Resolved<'a>>,
    graph: Graph<'a>,
}

impl<'a> Analyzer<'a> {
    pub fn new(files: Vec<&'a File<'a>>) -> Self {
        Self {
            taints: TaintList::new(),
            context: ContextStack::new(),
            files,
            resolved: HashMap::new(),
            graph: Graph::new(),
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
            graph: Graph::new(),
        }
    }

    /// first resolves names
    /// then begins traversal
    pub fn analyze(&mut self) -> String {
        for file in self.files.iter() {
            let cur = Cursor::from_file(file);
            self.resolved.extend(cur.resolve());
        }

        self.traverse(Cursor::from_file(
            self.files.get(0).expect("no files provided"),
        ));

        self.graph.dump()
    }

    /// traverse the program, looking for taints to trace, and following program flow
    /// Optionally returns a taint with the function
    fn traverse(&mut self, cursor: Cursor<'a>) -> bool {
        let mut returns = false;
        let mut closure = |cur: Cursor<'a>| -> Breaker {
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
                        if self.trace(cur) {
                            returns = true;
                        }
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
        returns
    }

    /// trace taints up the tree
    fn trace(&mut self, cursor: Cursor<'a>) -> bool {
        let mut path = Vec::new();
        let source = Taint::new_variable(cursor.clone());
        let mut returns = false;
        let mut index: usize = 0;
        let mut closure = |cur: Cursor<'a>| -> bool {
            match cur.kind() {
                "return_statement" => {
                    returns = true;
                    false
                }
                "expression_statement" => false,
                // record index
                "argument" => {
                    index = cur.get_index();
                    true
                }
                "assignment_expression" => {
                    let assign = Taint::new_variable(cur.clone());
                    self.taints.push(assign.clone());
                    path.push(cur);
                    self.graph.push(Vertex::new(
                        source.clone(),
                        self.context.clone(),
                        Some(assign),
                        path.clone(),
                    ));
                    false
                }
                "function_call_expression" => {
                    let mut cont = true;
                    if let Some(resolved) = self.resolved.clone().get(&cur.name().unwrap()) {
                        self.context.push(Context::new(
                            resolved.cursor().kind().to_string(),
                            resolved.cursor().name().unwrap(),
                        ));
                        let params = resolved.parameters();
                        let param_cur = params
                            .get(index)
                            .expect("unknown index (didnt pass through argument)");
                        let param_taint = Taint::new_variable(param_cur.clone());
                        self.taints.push(param_taint.clone());
                        path.push(cur);
                        self.graph.push(Vertex::new(
                            source.clone(),
                            self.context.clone(),
                            Some(param_taint.clone()),
                            path.clone(),
                        ));
                        cont = self.traverse(resolved.cursor());
                        self.taints.remove(&param_taint);
                        self.context.pop();
                    } else {
                        path.push(cur);
                    }
                    cont
                }
                _ => true,
            }
        };
        //     match cur.kind() {
        //         "return_statement" => {
        //             returns = true;
        //             false
        //         }
        //         "expression_statement" => false,
        //         // record index
        //         "argument" => {
        //             index = cur.get_index();
        //             true
        //         }
        //         "assignment_expression" => {
        //             let assign = Taint::new_variable(cur.clone());
        //             self.taints.push(assign.clone());
        //             path.push(cur.clone());
        //             self.graph.push(Vertex::new(
        //                 source.clone(),
        //                 self.context.clone(),
        //                 Some(assign),
        //                 path.clone(),
        //             ));
        //             false
        //         }
        //         "function_call_expression" => {
        //             let mut cont = true;
        //             let res_list = &self.resolved;
        //             if let Some(resolved) = &res_list.get(&cur.name().unwrap()) {
        //                 self.context.push(Context::new(
        //                     resolved.cursor().kind().to_string(),
        //                     resolved.cursor().name().unwrap(),
        //                 ));
        //                 let params = resolved.parameters();
        //                 let param_cur = params
        //                     .get(index)
        //                     .expect("unknown index (didnt pass through argument)");
        //                 let param_taint = Taint::new_variable(param_cur.clone());
        //                 self.taints.push(param_taint.clone());
        //                 cont = self.traverse(resolved.cursor());
        //                 self.taints.remove(&param_taint);
        //                 self.context.pop();
        //             }
        //             path.push(cur.clone());
        //             cont
        //         }
        //         _ => true,
        //     }
        let mut cursor = cursor;
        println!("{}, {:?}", cursor.kind(), cursor.name());
        cursor.trace(&mut closure);
        returns
    }
}
