use crate::graph::*;
use std::collections::HashSet;
use crate::node_to_string;
use crate::resolver;
use crate::resolver::*;
use crate::rules;
use tree_sitter::*;

// not same thing as context in last version
// this is to store hook/html stuff
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Context {
    kind: String,
    name: String,
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
            let vertex = Vertex::Source { tainting: taint };
            self.graph.push(vertex, None, None);
        }
    }

    fn build_graph(&mut self, start_file: &'a File<'a>) {
        let t = start_file.tree.clone();
        let mut cursor = t.walk();

        // start traversing with root of main file
        self.traverse_block(&mut cursor, &start_file);
    }

    fn traverse_block(&mut self, cursor: &mut TreeCursor, file: &'a File<'a>) {
        let start_node = cursor.node().id();
        let mut visited = false;
        loop {
            if visited {
                if cursor.goto_next_sibling() {
                    let cont = self.enter_node(&mut cursor.clone(), &file);
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
                    self.leave_node(&mut cursor.clone(), &file);
                    if cursor.node().id() == start_node {
                        break;
                    }
                } else {
                    break;
                }
            } else if cursor.goto_first_child() {
                self.enter_node(&mut cursor.clone(), &file);
            } else {
                visited = true;
            }
        }
    }

    // call with a cloned TreeCursor to not lose our place in the traversal
    fn enter_node(&mut self, cursor: &mut TreeCursor, file: &'a File<'a>) -> bool {
        let node = cursor.node();
        //println!("kind: {}", node.kind());
        match node.kind() {
            // return false to not crawl into these
            "function_definition" | "method_declaration" | "class_declaration" => return false,
            // these will find resolved funcs and crawl them too
            "function_call_expression" | "method_call_expression" => {
                let name = self.find_name(&mut cursor.clone(), &file);
                println!("found func {:?}", name);
                if let Ok(name) = name {
                    for f in self.files {
                        if let Some(resolved) = f.resolved.get(&name) {
                            match resolved {
                                Resolved::Function { name, cursor } => {
                                    if self.graphed_blocks.contains(name) {
                                        continue;
                                    }
                                    self.get_param_sources(&mut cursor.clone(), f);
                                    self.traverse_block(&mut cursor.clone(), f);
                                    self.graphed_blocks.insert(name.to_string());
                                }
                                _ => (),
                            }
                        }
                    }
                }
            }
            // if this is a taint, trace it
            "variable_name" => {
                let result = self.find_name(&mut cursor.clone(), &file);
                if let Ok(var_name) = &result {
                    for t in &self.taints.clone() {
                        if t.kind == "variable" {
                            if &t.name == var_name {
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
            }
            "if_statement" | "do_statement" | "for_statement" => {
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
        //println!("kind: {}", node.kind());
        match node.kind() {
            "if_statement" | "do_statement" | "for_statement" => {
                self.context_stack.pop();
            }
            _ => (),
        }
    }

    fn get_param_sources(&mut self, cursor: &mut TreeCursor, file: &'a File<'a>) -> Vec<Taint> {
        let start_node = cursor.node().id();
        let mut taints = Vec::new();
        let mut visited = false;
        loop {
            if visited {
                if cursor.goto_next_sibling() {
                    visited = false;
                    if cursor.node().kind() == "simple_parameter" {
                        let s = self.find_name(&mut cursor.clone(), file).expect("no name");
                        println!("taint name: {}", s.clone());
                        let taint = Taint {
                            kind: "param".to_string(),
                            name: s,
                            scope: self.current_scope(&mut cursor.clone(), &file),
                        };
                        println!("{:?}", self.current_scope(&mut cursor.clone(), &file));
                        taints.push(taint.clone());
                        self.taints.push(taint.clone());

                        let vertex = Vertex::Param { tainting: taint };
                        self.graph.push(vertex, None, None);
                    }
                } else if cursor.goto_parent() {
                    if cursor.node().id() == start_node {
                        break;
                    }
                } else {
                    break;
                }
            } else if cursor.goto_first_child() {
                if cursor.node().kind() == "simple_parameter" {
                    let s: String = self.find_name(&mut cursor.clone(), &file).expect("no name");
                    println!("taint name: {}", s.clone());
                    let taint = Taint {
                        kind: "source".to_string(),
                        name: s,
                        scope: self.current_scope(&mut cursor.clone(), &file),
                    };
                    taints.push(taint.clone());
                    self.taints.push(taint.clone());

                    let vertex = Vertex::Source { tainting: taint };
                    self.graph.push(vertex, None, None);
                }
            } else {
                visited = true;
            }
        }
        taints
    }

    fn find_name(&self, cursor: &mut TreeCursor, file: &'a File<'a>) -> Result<String, ()> {
        let mut visited = false;
        loop {
            if visited {
                if cursor.goto_next_sibling() {
                    visited = false;
                    if cursor.node().kind() == "name" {
                        let s: String = node_to_string(&cursor.node(), file.source_code);
                        return Ok(s);
                    }
                } else if cursor.goto_parent() {
                } else {
                    break;
                }
            } else if cursor.goto_first_child() {
                if cursor.node().kind() == "name" {
                    let s: String = node_to_string(&cursor.node(), file.source_code);
                    return Ok(s);
                }
            } else {
                visited = true;
            }
        }
        Err(())
    }

    fn trace_taint(&mut self, cursor: &mut TreeCursor, file: &'a File<'a>, parent_taint: Taint) {
        let arc = Arc {
            context_stack: self.context_stack.clone(),
        };
        let mut path: Vec<PathNode> = Vec::new();
        let mut vertex: Option<Vertex> = None;
        let mut child_taint: Option<Taint> = None;
        while cursor.goto_parent() {
            match cursor.node().kind() {
                "assignment_expression" => {
                    if let Ok(name) = self.find_name(&mut cursor.clone(), &file) {
                        if (name == parent_taint.name) {
                            break;
                        }
                        child_taint = Some(Taint {
                            kind: "variable".to_string(),
                            name,
                            scope: self.current_scope(&mut cursor.clone(), &file),
                        });
                        vertex = Some(Vertex::Assignment {
                            parent_taint: parent_taint.clone(),
                            kind: "assignment_expression".to_string(),
                            tainting: child_taint.clone().expect("eeeerrrrooorrr"),
                            path: path.clone(),
                        });
                        break;
                    }
                }
                "function_call_expression" => {
                    let name = self.find_name(&mut cursor.clone(), &file);
                    if let Ok(name) = name {
                        let mut cont = true;
                        for f in self.files {
                            if let Some(resolved) = f.resolved.get(&name) {
                                cont = false;
                                match resolved {
                                    Resolved::Function { name, cursor } => {
                                        println!("setting resolved vertex{}", name.to_string());
                                        path.push(PathNode::Resolved { name: name.clone() });
                                        vertex = Some(Vertex::Resolved {
                                            parent_taint: parent_taint.clone(),
                                            name: name.to_string(),
                                            path: path.clone(),
                                        });
                                    }
                                    _ => (),
                                }
                            }
                        }
                        if cont {
                            path.push(PathNode::Unresolved { name: name.clone() });
                            vertex = Some(Vertex::Unresolved {
                                parent_taint: parent_taint.clone(),
                                name: name.to_string(),
                                path: path.clone(),
                            });
                        }
                    }
                }
                "method_call_expression" => {
                    let name = self.find_name(&mut cursor.clone(), &file);
                    if let Ok(name) = name {
                        path.push(PathNode::Unresolved { name: name.clone() });
                        vertex = Some(Vertex::Unresolved {
                            parent_taint: parent_taint.clone(),
                            name,
                            path: path.clone(),
                        });
                    }
                }
                "echo_statement" => {
                    // sink
                    // break
                }
                _ => (),
            }
        }
        if let Some(taint) = child_taint {
            self.taints.push(taint);
        }
        if let Some(vertex) = vertex {
            self.graph
                .push(vertex, Some(arc), Some(parent_taint.clone()));
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
                    let name = self.find_name(&mut cursor.clone(), &file).expect("no name");
                    scope.function = Some(name);
                }
                "class_declaration" => {
                    let name = self.find_name(&mut cursor.clone(), &file).expect("no name");
                    scope.class = Some(name);
                }
                _ => (),
            }
        }

        println!("current scope: {:?}", scope);

        scope
    }
}
