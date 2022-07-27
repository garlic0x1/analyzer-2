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

    /// first resolves names
    /// then begins traversal
    pub fn analyze(&mut self) {
        self.resolve_files();

        let mut cursors = Vec::new();
        for file in self.files.iter() {
            let cur = Cursor::from_file(file);
            cursors.push(cur);
        }

        for cur in cursors {
            self.traverse(cur);
        }
    }

    /// returns graph ( you must run analyze() first to populate it )
    pub fn graph(&'a self) -> &'a Graph<'a> {
        &self.graph
    }

    /// traverse the program, looking for taints to trace, and following program flow
    /// Optionally returns a taint with the function
    fn traverse(&mut self, cursor: Cursor<'a>) -> bool {
        let mut returns = false;
        let mut traversal =
            Traversal::new_block(&cursor, vec!["method_declaration", "function_definition"]);
        while let Some(motion) = traversal.next() {
            match motion {
                Order::Enter(cur) => {
                    match cur.kind() {
                        "variable_name" => {
                            if let Some(taint) = self.get_taint(cur.clone()) {
                                if self.trace(cur.clone(), taint) {
                                    returns = true;
                                }
                            }
                        }
                        // push context
                        "if_statement" => {
                            self.context
                                .push(Context::new(cur.kind().to_string(), cur.kind().to_string()));
                        }
                        _ => (),
                    }
                }
                Order::Leave(cur) => {
                    match cur.kind() {
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
        //let mut path = vec![cursor.clone()];
        let mut path = Vec::new();
        let mut index: usize = 0;

        let mut tracer = Trace::new(cursor);
        while let Some(cur) = tracer.next() {
            if let Some(s) = cur.field() {
                if s == "condition" {
                    break;
                }
            }
            match cur.kind() {
                "return_statement" => {
                    let assign = Taint::new_return(cur.clone());
                    path.push(cur.clone());
                    self.push_taint(cur.clone(), source.clone(), assign, path.clone());
                    return true;
                }
                "assignment_expression" => {
                    let assign = Taint::new_variable(cur.clone());
                    path.push(cur.clone());
                    self.push_taint(cur.clone(), source.clone(), assign, path.clone());
                    return false;
                }
                "function_call_expression"
                | "member_call_expression"
                | "scoped_call_expression" => {
                    path.push(cur.clone());
                    self.call(cur, Some(index), Some(source.clone()), Some(path.clone()));
                }
                "echo_statement" => path.push(cur),
                "argument" => index = cur.get_index(),
                //"expression_statement" => break,
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
    fn call(
        &mut self,
        cursor: Cursor<'a>,
        index: Option<usize>,
        source: Option<Taint>,
        path: Option<Vec<Cursor<'a>>>,
    ) {
        let name = match cursor.name() {
            Some(name) => name,
            None => cursor.to_string().replace("\"", "").replace("'", ""),
        };
        if let Some(resolved) = self.resolved.clone().get(&name) {
            // passing taint into param
            if let Some(index) = index {
                if let Some(param_cur) = resolved.parameters().get(index) {
                    if self.context.push(Context::new(
                        resolved.cursor().kind().to_string(),
                        resolved.cursor().name().unwrap(),
                    )) {
                        // push taint
                        self.push_taint(
                            param_cur.clone(),
                            source.unwrap(),
                            Taint::new_param(param_cur.clone()),
                            path.unwrap(),
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
                // simple jump dont pass a taint
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
    }

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

    // create taint and graph it
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
