use crate::analyzer::taint::*;
use crate::graph::flowgraph::*;
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
            taints.push(Taint::new_source(source));
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
        let mut closure = |cur: Cursor<'a>, entering: bool| -> Breaker {
            if entering {
                match cur.kind() {
                    "variable_name" => {
                        // check if in left of assignment and return
                        if let Some(s) = cur.raw_cursor().field_name() {
                            if s == "left" {
                                return Breaker::Continue;
                            }
                        }
                        // check for taint and trace
                        if let Some(t) = self.taints.get(&Taint::new_variable(cur.clone())) {
                            if self.trace(cur, t) {
                                returns = true;
                            }
                        }

                        Breaker::Continue
                    }
                    // do not crawl into these node types
                    "function_definition" | "method_declaration" => Breaker::Pass,
                    // push context
                    "if_statement" => {
                        self.context
                            .push(Context::new(cur.kind().to_string(), cur.kind().to_string()));
                        Breaker::Continue
                    }
                    _ => Breaker::Continue,
                }
            } else {
                match cur.kind() {
                    "function_call_expression" | "member_call_expression" => {
                        for t in self.taints.returns().iter() {
                            if self.trace(cur.clone(), t.clone()) {
                                returns = true;
                            }
                        }
                        Breaker::Continue
                    }
                    "if_statement" => {
                        self.context.pop();
                        Breaker::Continue
                    }
                    _ => Breaker::Continue,
                }
            }
        };
        let mut cursor = cursor.clone();
        cursor.traverse(&mut closure);
        returns
    }

    /// trace taints up the tree
    fn trace(&mut self, cursor: Cursor<'a>, source: Taint) -> bool {
        let mut path = Vec::new();
        let mut has_return = false;
        let mut push_path = false;
        let mut index: usize = 0;
        let mut closure = |cur: Cursor<'a>| -> bool {
            match cur.kind() {
                "return_statement" => {
                    let assign = Taint::new_return(cur.clone());
                    path.push(cur.clone());
                    self.push_taint(cur.clone(), source.clone(), assign.clone(), path.clone());
                    has_return = true;
                    push_path = false;
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
                    path.push(cur.clone());
                    self.push_taint(cur.clone(), source.clone(), assign.clone(), path.clone());
                    push_path = false;
                    false
                }
                "function_call_expression" | "member_call_expression" => {
                    let mut cont = true;
                    if let Some(resolved) = self.resolved.clone().get(&cur.name().unwrap()) {
                        self.context.push(Context::new(
                            resolved.cursor().kind().to_string(),
                            resolved.cursor().name().unwrap(),
                        ));
                        let params = resolved.parameters();
                        println!("{:?}", params.len());
                        let param_cur = params
                            .get(index)
                            .expect("unknown index (didnt pass through argument)");

                        println!("{:?}", param_cur.name());
                        let param_taint = Taint::new_param(param_cur.clone());
                        path.push(cur.clone());

                        //push
                        self.push_taint(
                            param_cur.clone(),
                            source.clone(),
                            param_taint.clone(),
                            path.clone(),
                        );

                        // traverse and see if it has tainted return
                        cont = self.traverse(resolved.cursor());

                        //pop
                        self.taints.clear_scope(&Scope::new(param_cur.clone()));
                        self.graph.clear_scope(&Scope::new(param_cur.clone()));
                        self.context.pop();

                        if cont {
                            for ret in self.taints.returns() {
                                self.trace(cur.clone(), ret);
                            }
                        }
                        self.taints.clear_returns();
                        self.graph.clear_returns();
                        false
                    } else {
                        path.push(cur);
                        push_path = true;
                        true
                    }
                }
                _ => true,
            }
        };
        let mut cursor = cursor;
        cursor.trace(&mut closure);
        if push_path {
            if let Some(cur) = path.pop() {
                let vert = Vertex::new(source, self.context.clone(), None, path);
                self.graph.push(cur, vert);
            }
        }

        has_return
    }

    fn push_taint(&mut self, cur: Cursor<'a>, source: Taint, assign: Taint, path: Vec<Cursor<'a>>) {
        println!("@@@");
        self.taints.push(assign.clone());
        self.graph.push(
            cur.clone(),
            Vertex::new(
                source.clone(),
                self.context.clone(),
                Some(assign),
                path.clone(),
            ),
        );
    }
}
