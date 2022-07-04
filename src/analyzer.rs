use crate::graph::*;
use crate::node_to_string;
use crate::resolver::*;
use crate::rules;
use std::collections::HashSet;
use tree_sitter::*;

// not same thing as context in last version
// this is to store hook/html stuff
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Context {
    pub kind: String,
    pub name: String,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Taint {
    pub kind: String,
    pub name: String,
    pub scope: Scope,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Scope {
    pub filename: Option<String>,
    pub class: Option<String>,
    pub function: Option<String>,
}

pub struct Analyzer<'a> {
    graphed_blocks: HashSet<String>,
    files: &'a Vec<File<'a>>,
    rules: rules::Rules,
    pub graph: Graph,
    context_stack: Vec<Context>,
    taints: Vec<Taint>,
}

impl<'a> Analyzer<'a> {
    pub fn new(files: &'a Vec<File<'a>>, rules: rules::Rules) -> Self {
        let mut s = Self {
            files,
            rules,
            graphed_blocks: HashSet::new(),
            graph: Graph::new(),
            taints: Vec::new(),
            context_stack: Vec::new(),
        };
        // initialize the taint vec with sources of input
        let start_file_reference: &'a File<'a> = &s.files[0];
        s.load_sources();
        s.build_graph(&start_file_reference);
        return s;
    }

    fn load_sources(&mut self) {
        for source in self.rules.sources.iter() {
            let taint = Taint {
                kind: "source".to_string(),
                name: source.name.clone(),
                scope: Scope {
                    filename: None,
                    class: None,
                    function: None,
                },
            };
            self.taints.push(taint.clone());
            let vertex = Vertex::Source {
                tainting: taint,
                context_stack: self.context_stack.clone(),
            };
            self.graph.push(vertex);
        }
    }

    fn build_graph(&mut self, start_file: &'a File<'a>) {
        let t = start_file.tree.clone();
        let mut cursor = t.walk();

        // start traversing with root of main file
        self.graph(&mut cursor, &start_file, &mut None);
    }

    fn graph(&mut self, cursor: &mut TreeCursor, file: &'a File<'a>, local_taint: &Option<Taint>) {
        let start_node = cursor.node().id();
        let mut visited = false;
        let mut stack = Vec::new();
        loop {
            if visited {
                if cursor.goto_next_sibling() {
                    let inc = stack.pop().expect("empty stack");
                    stack.push(inc + 1);
                    let cont = self.enter_node(&mut cursor.clone(), inc, file, &local_taint);
                    if !cont {
                        if cursor.goto_next_sibling() {
                            visited = false;
                            continue;
                        } else if cursor.goto_parent() {
                            visited = true;
                            continue;
                        }
                    }
                    visited = false;
                } else if cursor.goto_parent() {
                    stack.pop();
                    self.leave_node(&mut cursor.clone(), file);
                    if cursor.node().id() == start_node {
                        break;
                    }
                } else {
                    break;
                }
            } else if cursor.goto_first_child() {
                stack.push(0);
                self.enter_node(&mut cursor.clone(), 0, file, local_taint);
            } else {
                visited = true;
            }
        }
    }

    // call with a cloned TreeCursor to not lose our place in the traversal
    fn enter_node(&mut self, cursor: &mut TreeCursor, index: u32, file: &'a File<'a>, local_taint: &Option<Taint>) -> bool {
        let node = cursor.node();
        ////println!("Node: {}", node.kind());
        match node.kind() {
            // return false to not crawl into these
            "function_definition" | "method_declaration" | "class_declaration" => return false,
            // these will find resolved funcs and crawl them too
            // if this is a taint, trace it
            "variable_name" => {
                if let Some(s) = cursor.field_name() {
                    if s == "left" {
                        return true;
                    }
                }
                let result = file.find_name(&mut cursor.clone());
                if let Some(t) = local_taint.clone() {
                    self.taints.push(t);
                }
                if let Some(var_name) = result {
                    for t in &self.taints.clone() {
                        if t.kind == "variable" {
                            if t.name == var_name {
                                let taint = t.clone();
                                self.trace_taint(cursor, &file, taint);
                            }
                        } else if t.kind == "source" {
                            if t.name.as_str() == var_name.clone() {
                                let taint = t.clone();

                                self.trace_taint(cursor, &file, taint);
                            }
                        } else if t.kind == "param" {
                            if t.name.as_str() == var_name.clone() {
                                let taint = t.clone();

                                self.trace_taint(cursor, &file, taint);
                            }
                        }
                    }
                }
                if let Some(t) = local_taint.clone() {
                    self.taints.pop();
                }
            }
            "if_statement" | "do_statement" | "for_statement" => {
                //println!("aaa {:?}", node.kind());
                self.context_stack.push(Context {
                    kind: node.kind().to_string(),
                    name: node_to_string(&node, &file.source_code),
                });
            }
            _ => return true,
        }
        return true;
    }

    fn leave_node(&mut self, cursor: &mut TreeCursor, file: &'a File<'a>) {
        let node = cursor.node();
        match node.kind() {
            "if_statement" | "do_statement" | "for_statement" => {
                self.context_stack.pop();
            }
            "function_call_expression" | "method_call_expression" => {
                let name = file.find_name(&mut cursor.clone());
                if let Some(name) = name {
                    for f in self.files {
                        if let Some(resolved) = f.resolved.get(&name) {
                            match resolved {
                                Resolved::Function { name, cursor, .. } => {
                                    if self.graphed_blocks.contains(name) {
                                        continue;
                                    }
                                    self.context_stack.push(Context {
                                        kind: node.kind().to_string(),
                                        name: node_to_string(&node, &f.source_code),
                                    });
                                    self.graph(&mut cursor.clone(), f, &mut None);
                                    self.context_stack.pop();
                                    self.graphed_blocks.insert(name.to_string());
                                }
                                _ => (),
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }

    fn find_index(&mut self, cursor: &mut TreeCursor) -> usize {
        let arg_id = cursor.node().id();
        let mut index = 0;
        cursor.goto_parent();
        cursor.goto_first_child();
        while cursor.node().id() != arg_id {
            if cursor.node().is_named() {
                index += 1;
            }
            cursor.goto_next_sibling();
        }

        index
    }

    fn trace_taint(&mut self, cursor: &mut TreeCursor, file: &'a File<'a>, parent_taint: Taint) {
        println!("tracing taint: {:?}", parent_taint);
        let mut path: Vec<PathNode> = Vec::new();
        let mut vertex: Option<Vertex> = None;
        let mut child_taint: Option<Taint> = None;
        let mut index = 0;
        while cursor.goto_parent() {
            match cursor.node().kind() {
                "argument" => {
                    index = self.find_index(&mut cursor.clone());
                }
                "function_call_expression" => {
                    let save_cursor = cursor.clone();
                    let name = file.find_name(&mut cursor.clone());
                    let mut tainted_return = false;
                    if let Some(name) = name {
                        let mut is_resolved = false;
                        for f in self.files {
                            if let Some(resolved) = f.resolved.get(&name) {
                                is_resolved = true;
                                match resolved {
                                    Resolved::Function {
                                        name,
                                        cursor,
                                        params,
                                    } => {
                                        path.push(PathNode::Resolved { name: name.clone() });
                                        vertex = Some(Vertex::Resolved {
                                            parent_taint: parent_taint.clone(),
                                            name: name.to_string(),
                                            path: path.clone(),
                                            context_stack: self.context_stack.clone(),
                                        });
                                        let taint = Taint {
                                            kind: "param".to_string(),
                                            name: params.get(index).unwrap().to_string(),
                                            scope: Scope {
                                                filename: Some(f.filename.to_string()),
                                                function: Some(name.to_string()),
                                                class: None,
                                            },
                                        };
                                        self.graph.push(vertex.clone().unwrap());
                                        vertex = None;
                                        self.graph.push(Vertex::Param {
                                            tainting: taint.clone(),
                                            context_stack: self.context_stack.clone(),
                                        });
                                        self.context_stack.push(Context {
                                            kind: save_cursor.node().kind().to_string(),
                                            name: node_to_string(
                                                &save_cursor.node(),
                                                &file.source_code,
                                            ),
                                        });
                                        self.graphed_blocks.insert(name.to_string());
                                        self.graph(&mut cursor.clone(), f, &mut Some(taint));
                                        self.context_stack.pop();

                                    }
                                    _ => (),
                                }
                            }
                        }
                        if !is_resolved {
                            path.push(PathNode::Unresolved { name: name.clone() });
                            vertex = Some(Vertex::Unresolved {
                                parent_taint: parent_taint.clone(),
                                name: name.to_string(),
                                path: path.clone(),
                                context_stack: self.context_stack.clone(),
                            });
                        } else {
                            break;
                        }
                    }
                }
                "method_call_expression" => {
                    let name = file.find_name(&mut cursor.clone());
                    if let Some(name) = name {
                        path.push(PathNode::Unresolved { name: name.clone() });
                        vertex = Some(Vertex::Unresolved {
                            parent_taint: parent_taint.clone(),
                            name: name.to_string(),
                            path: path.clone(),
                            context_stack: self.context_stack.clone(),
                        });
                    }
                }
                "assignment_expression" => {
                    if let Some(name) = file.find_name(&mut cursor.clone()) {
                        child_taint = Some(Taint {
                            kind: "variable".to_string(),
                            name: name.clone(),
                            scope: self.current_scope(&mut cursor.clone(), &file),
                        });
                        vertex = Some(Vertex::Assignment {
                            parent_taint: parent_taint.clone(),
                            kind: "assignment_expression".to_string(),
                            tainting: child_taint.clone().expect("eeeerrrrooorrr"),
                            path: path.clone(),
                            context_stack: self.context_stack.clone(),
                        });
                        break;
                    }
                }
                "echo_statement" => {
                    // sink
                    // break
                }
                "formal_parameters" | "simple_parameter" => {
                    return;
                }
                _ => (),
            }
        }
        if let Some(taint) = child_taint {
            self.taints.push(taint);
        }
        if let Some(vertex) = vertex {
            self.graph.push(vertex);
        }
    }

    fn current_scope(&self, cursor: &mut TreeCursor, file: &File) -> Scope {
        let mut scope = Scope {
            filename: Some(file.filename.clone()),
            function: None,
            class: None,
        };

        while cursor.goto_parent() {
            match cursor.node().kind() {
                "function_definition" | "method_declaration" => {
                    let name = file.find_name(&mut cursor.clone());
                    scope.function = Some(name.unwrap().to_string());
                }
                "class_declaration" => {
                    let name = file.find_name(&mut cursor.clone());
                    scope.class = Some(name.unwrap().to_string());
                }
                _ => (),
            }
        }

        scope
    }
}
