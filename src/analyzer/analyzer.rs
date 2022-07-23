use crate::analyzer::taint::*;
use crate::graph::flowgraph::*;
use crate::graph::rules::*;
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
    pub fn new(files: Vec<&'a File>, rules: Rules) -> Self {
        let mut taints = TaintList::new();
        let mut hooks = HashSet::new();
        for source in rules.sources() {
            taints.push(Taint::new_source(source.to_string()));
        }
        for hook in rules.hooks() {
            hooks.insert(hook.to_string());
        }

        Self {
            taints: TaintList::new(),
            context: ContextStack::new(),
            files,
            resolved: HashMap::new(),
            graph: Graph::new(),
            hooks,
        }
    }

    /// returns a new analyzer with sources to trace
    pub fn from_sources(files: Vec<&'a File>, sources: Vec<String>) -> Self {
        let mut taints = TaintList::new();
        for source in sources {
            taints.push(Taint::new_source(source));
        }
        let hooks: HashSet<String> = vec!["add_action".to_string()].into_iter().collect();

        Self {
            files,
            taints,
            context: ContextStack::new(),
            resolved: HashMap::new(),
            graph: Graph::new(),
            hooks,
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
                            // check if in left of assignment and return
                            if let Some(s) = cur.field() {
                                if s == "left" || s == "object" {
                                    continue;
                                }
                            }
                            // check for taint and trace
                            if let Some(t) = self.taints.get(&Taint::new_variable(cur.clone())) {
                                let cur_scope = Scope::new(cur.clone());
                                eprintln!("testing {:?}", t);
                                if cur_scope.contains(&t.scope) {
                                    if self.trace(cur.clone(), t) {
                                        returns = true;
                                    }
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
                        "function_call_expression" | "member_call_expression" => {
                            /*
                            for t in self.taints.returns().iter() {
                                if self.trace(cur.clone(), t.clone()) {
                                    returns = true;
                                }
                            }
                            */
                            self.jump(&cur.name().unwrap());
                            // if not recursive, jump
                            if self.hooks.contains(&cur.name().unwrap()) {
                                self.handle_hook(cur);
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

    /// push ctx to stack and enter new frame, returns true if there are taints.
    fn jump(&mut self, name: &str) {
        if let Some(resolved) = self.resolved.clone().get(name) {
            if self.context.push(Context::new(
                resolved.cursor().kind().to_string(),
                resolved.name(),
            )) {
                eprintln!("jumping to {}", resolved.name());
                let mut cur = resolved.cursor();
                println!("found body: {}", cur.goto_field("body"));
                self.traverse(cur);
                self.context.pop();
            }
        }
    }

    fn handle_hook(&mut self, cursor: Cursor<'a>) {
        for motion in cursor.traverse() {
            if let Order::Enter(cur) = motion {
                if cur.kind() == "argument" {
                    if cur.to_string().len() > 2 {
                        let name = &cur.to_string()[1..cur.to_string().len() - 1];
                        self.jump(name);
                    }
                }
            }
        }
    }

    /// trace taints up the tree
    fn trace(&mut self, cursor: Cursor<'a>, source: Taint) -> bool {
        let mut path = Vec::new();
        let mut has_return = false;
        let mut push_path = false;
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
                    has_return = true;
                    push_path = false;
                    break;
                }
                "expression_statement" => break,
                // record index
                "argument" => {
                    index = cur.get_index();
                }
                "assignment_expression" => {
                    let assign = Taint::new_variable(cur.clone());
                    path.push(cur.clone());
                    self.push_taint(cur.clone(), source.clone(), assign, path.clone());
                    push_path = false;
                    break;
                }
                "function_call_expression" | "member_call_expression" => {
                    if let Some(resolved) = self.resolved.clone().get(&cur.name().unwrap()).clone()
                    {
                        let resolved = resolved.clone();
                        let params = resolved.parameters().clone();
                        let param_cur = params
                            .get(index)
                            .expect(&format!(
                                "cant find param {} {}",
                                cur.to_string(),
                                source.name,
                            ))
                            .clone();

                        let param_taint = Taint::new_param(param_cur.clone());
                        path.push(cur.clone());

                        if !self.context.push(Context::new(
                            resolved.cursor().kind().to_string(),
                            resolved.cursor().name().unwrap(),
                        )) {
                            continue;
                        }

                        //push
                        self.push_taint(
                            param_cur.clone(),
                            source.clone(),
                            param_taint,
                            path.clone(),
                        );
                        // traverse and see if it has tainted return
                        let mut res_cur = resolved.cursor();
                        res_cur.goto_field("body");
                        let cont = self.traverse(res_cur);

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
                        break;
                    } else {
                        path.push(cur);
                        push_path = true;
                    }
                }
                // special sinks
                "echo_statement" => {
                    path.push(cur);
                    push_path = true;
                    break;
                }
                _ => (),
            }
        }
        if push_path {
            if let Some(cur) = path.clone().last() {
                let pitem = PathItem::new(source.clone(), path);
                let vert = Vertex::new(self.context.clone(), None, pitem);
                self.graph.push(cur.clone(), vert);
            }
        }

        has_return
    }

    fn push_taint(&mut self, cur: Cursor<'a>, source: Taint, assign: Taint, path: Vec<Cursor<'a>>) {
        self.taints.push(assign.clone());
        let pitem = PathItem::new(source.clone(), path);
        self.graph.push(
            cur.clone(),
            Vertex::new(self.context.clone(), Some(assign.clone()), pitem),
        );
    }

    fn resolve_files(&mut self) {
        for file in self.files.iter() {
            for motion in file.traverse() {
                if let Order::Enter(cur) = motion.clone() {
                    match cur.kind() {
                        "function_definition" | "method_declaration" => {
                            self.resolved
                                .insert(cur.name().unwrap(), Resolved::new_function(cur));
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
