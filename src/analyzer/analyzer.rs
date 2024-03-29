use crate::analyzer::taint::*;
use crate::graph::graph::*;
use crate::graph::rules::*;
use crate::graph::vertex::*;
use crate::tree::cursor::*;
use crate::tree::file::*;
use crate::tree::resolved::*;
use crate::tree::tracer::Trace;
use crate::tree::traverser::*;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct Analyzer<'a> {
    taints: TaintList,
    context: ContextStack,
    files: Vec<&'a File>,
    resolved: HashMap<String, Resolved<'a>>,
    graph: Graph<'a>,
    hooks: HashSet<String>,
}

impl<'a> Analyzer<'a> {
    pub fn new(files: Vec<&'a File>, ruleset: &Rules) -> Self {
        let mut taints = TaintList::new();
        for source in ruleset.sources().iter() {
            taints.push(Taint::new_source(source.to_string()));
        }

        Self {
            files,
            taints,
            context: ContextStack::new(),
            resolved: HashMap::new(),
            graph: Graph::new(),
            hooks: ruleset.hooks().clone(),
        }
    }

    /// analyze tree and produce a flow graph
    pub fn graph(&mut self) -> &Graph<'a> {
        self.resolve_files();

        // crawl each file ( since with wordpress these can sometimes still be accessed )
        for file in self.files.clone() {
            self.traverse(Cursor::from_file(file));
        }

        // return graph for applying rules
        &self.graph
    }

    /// traverse the program, looking for taints to trace, and following program flow
    /// Optionally returns a taint with the function
    fn traverse(&mut self, cursor: Cursor<'a>) -> bool {
        let mut returns = false;

        let mut traversal =
            Traversal::new_block(&cursor, vec!["method_declaration", "function_definition"]);

        // depth first iterator that returns enum Order { Enter, Leave }
        while let Some(motion) = traversal.next() {
            match motion {
                // push context
                Order::Enter(cur) => match cur.kind() {
                    "if_statement" => {
                        self.context
                            .push(Context::new(cur.kind().to_string(), cur.kind().to_string()));
                    }
                    _ => (),
                },
                // pop context and trace taints
                Order::Leave(cur) => {
                    match cur.kind() {
                        // trace if taint
                        "variable_name" => {
                            if let Some(taint) = self.get_taint(cur.clone()) {
                                if self.trace(cur.clone(), taint) {
                                    returns = true;
                                }
                            }
                        }
                        // call function
                        "function_call_expression"
                        | "member_call_expression"
                        | "scoped_call_expression" => {
                            if let Some(n) = cur.name() {
                                self.call(cur.clone(), None, None, None);
                                // if not recursive, jump
                                if self.hooks.contains(&n) {
                                    self.handle_hook(cur);
                                }
                            }
                        }
                        // pop context
                        "if_statement" => {
                            self.context.pop();
                        }
                        _ => (),
                    }
                }
            }
        }

        returns
    }

    /// trace taints up the tree
    fn trace(&mut self, cursor: Cursor<'a>, source: Taint) -> bool {
        let mut path = Vec::new();
        let mut index: usize = 0;
        let mut tracer = Trace::new(cursor);
        while let Some(cur) = tracer.next() {
            // dont trace through boolean conditions
            if let Some(s) = cur.field() {
                if s == "condition" {
                    break;
                }
            }
            match cur.kind() {
                // these vertices propagate taints
                "return_statement" | "assignment_expression" => {
                    if let Ok(assign) = Taint::from_trace(cur.clone()) {
                        path.push(cur.clone());
                        self.push_taint(cur.clone(), source.clone(), assign, path.clone());
                        return true;
                    }
                }

                // these need to be recorded as possible sanitizers
                "cast_expression" => {
                    let mut type_node = cur.clone();
                    type_node.goto_field("type");
                    println!("cast expr {}", type_node.kind());
                    path.push(type_node);
                }

                // these can be unresolved or resolved
                // unresolved can be sanitizers
                // resolved can propogate taint to params
                "function_call_expression"
                | "member_call_expression"
                | "scoped_call_expression" => {
                    path.push(cur.clone());
                    if !self.call(cur, Some(index), Some(source.clone()), Some(path.clone())) {
                        break;
                    }
                }

                // this is a special sink for PHP
                "echo_statement" => path.push(cur),

                // keep track of index to know which params we might need to taint
                "argument" => index = cur.get_index(),

                // data doesnt flow up from an expression statement
                "expression_statement" => break,
                _ => (),
            }
        }
        if let Some(cur) = path.clone().last() {
            let pitem = PathItem::new(source.clone(), path);
            let vert = Vertex::new(None, self.context.clone());
            self.graph.push(pitem, cur.clone(), vert);
        }

        false
    }

    /// push ctx to stack and enter new frame, returns true if there are taints.
    /// returns true if simulated return is tainted
    fn call(
        &mut self,
        cursor: Cursor<'a>,
        index: Option<usize>,
        source: Option<Taint>,
        path: Option<Vec<Cursor<'a>>>,
    ) -> bool {
        let mut passes_taint = true;

        // figure out the name of the function
        let name = match cursor.name() {
            Some(name) => name,
            None => cursor.to_string().replace("\"", "").replace("'", ""),
        };

        // confirm function is a resolved one
        if let Some(resolved) = self.resolved.clone().get(&name) {
            passes_taint = false;
            // passing taint into param
            if let (Some(index), Some(source), Some(path)) = (index, source, path) {
                if let Some(param_cur) = resolved.parameters().get(index) {
                    // if graph tells us to continue
                    if self.context.push(Context::new(
                        resolved.cursor().kind().to_string(),
                        resolved.cursor().name().unwrap_or_default(),
                    )) {
                        // push taint
                        self.push_taint(
                            param_cur.clone(),
                            source,
                            Taint::new_param(param_cur.clone()),
                            path,
                        );

                        // traverse and see if it has tainted return
                        let mut res_cur = resolved.cursor();
                        res_cur.goto_field("body");
                        let cont = self.traverse(res_cur);
                        self.context.pop();

                        // clear local taints and graph leaves
                        self.taints.clear_scope(&Scope::new(param_cur.clone()));
                        self.graph.clear_scope(&Scope::new(param_cur.clone()));

                        // trace from here if taint is returned
                        if cont {
                            for ret in self.taints.returns() {
                                self.trace(cursor.clone(), ret);
                            }
                        }

                        // clear return taints and graph leaves
                        self.taints.clear_returns();
                        self.graph.clear_returns();
                    }
                }
            } else {
                // simple jump dont pass a taint or clear after
                // (all calls to that block should taint same stuff with no input)

                // if graph tells us to continue
                if self.context.push(Context::new(
                    resolved.cursor().kind().to_string(),
                    resolved.name(),
                )) {
                    let mut res_cur = resolved.cursor();
                    res_cur.goto_field("body");
                    self.traverse(res_cur);
                    self.context.pop();
                }
            }
        }

        passes_taint
    }

    /// get a taint associated with this cursor
    fn get_taint(&self, cursor: Cursor<'a>) -> Option<Taint> {
        match cursor.kind() {
            "variable_name" => {
                // check if in left of assignment and return
                if let Some(s) = cursor.field() {
                    if s == "left" {
                        let mut pcur = cursor.clone();
                        pcur.goto_parent();
                        if pcur.kind() == "assignment_expression" {
                            return None;
                        }
                    }
                    if s == "object" {
                        return None;
                    }
                }
                // check for taint
                if let Some(taint) = self.taints.get(&Taint::new_variable(cursor.clone())) {
                    // check if in scope
                    let cur_scope = Scope::new(cursor.clone());
                    if cur_scope.contains(&taint.scope) {
                        return Some(taint);
                    }
                }

                None
            }
            _ => None,
        }
    }

    /// call functions that are hooked
    fn handle_hook(&mut self, cursor: Cursor<'a>) {
        let mut cursor = cursor;
        cursor.goto_field("arguments");
        let mut traversal = Traversal::new(&cursor);
        while let Some(motion) = traversal.next() {
            if let Order::Enter(cur) = motion {
                if cur.kind() == "argument" {
                    if cur.to_string().len() > 2 {
                        self.call(cur.clone(), None, None, None);
                    }
                }
            }
        }
    }

    /// create taint and graph it
    fn push_taint(&mut self, cur: Cursor<'a>, source: Taint, assign: Taint, path: Vec<Cursor<'a>>) {
        self.taints.push(assign.clone());
        let pitem = PathItem::new(source.clone(), path);
        self.graph
            .push(pitem, cur, Vertex::new(Some(assign), self.context.clone()));
    }

    /// load resolved items to analyzer
    fn resolve_files(&mut self) {
        for file in self.files.iter() {
            for motion in file.traverse() {
                if let Order::Enter(cur) = motion.clone() {
                    match cur.kind() {
                        "function_definition" | "method_declaration" => {
                            if let Some(n) = cur.name() {
                                self.resolved.insert(n, Resolved::new_function(cur));
                            }
                        }
                        "program" => {
                            self.resolved
                                .insert("ROOT".to_string(), Resolved::new_root(cur));
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}
