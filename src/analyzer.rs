use crate::graph::*;
use crate::node_to_string;
use crate::resolver;
use crate::resolver::*;
use crate::rules;
use tree_sitter::*;

// not same thing as context in last version
// this is to store hook/html stuff
#[derive(Clone, Debug)]
pub struct Context {
    kind: String,
    name: String,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Taint<'a> {
    pub kind: String,
    pub name: String,
    pub scope: Scope<'a>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Scope<'a> {
    pub global: bool,
    pub file: Option<&'a File<'a>>,
    pub class: Option<String>,
    pub function: Option<String>,
}

pub struct Analyzer<'a> {
    files: Vec<resolver::File<'a>>,
    rules: rules::Rules,
    pub graph: Graph<'a>,
    context_stack: Vec<Context>,
    taints: Vec<Taint<'a>>,
    current_scope: Scope<'a>,
}
impl<'a> Analyzer<'a> {
    pub fn new(files: Vec<File<'a>>, rules: rules::Rules) -> Self {
        let mut s = Self {
            files: files.clone(),
            rules,
            current_scope: Scope {
                global: false,
                file: None,
                class: None,
                function: None,
            },
            graph: Graph::new(),
            taints: Vec::new(),
            context_stack: Vec::new(),
        };
        // initialize the taint vec with sources of input
        s.load_sources();
        return s;
    }

    fn load_sources(&mut self) {
        for source in self.rules.sources.iter() {
            let taint = Taint {
                kind: "source".to_string(),
                name: source.name.clone(),
                scope: Scope {
                    global: true,
                    file: None,
                    class: None,
                    function: None,
                },
            };
            self.taints.push(taint);
        }
    }

    pub fn build_graph(&mut self) {
        let file = self.files.get(0).expect("no files").clone(); // start with first (assumed main for now) file
        let t = file.tree.clone();
        let mut cursor = t.walk();

        // start traversing with root of main file
        self.traverse_block(&mut cursor, &file);
    }

    fn traverse_block(&mut self, cursor: &mut TreeCursor, file: &File) {
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
    fn enter_node(&mut self, cursor: &mut TreeCursor, file: &File) -> bool {
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
                    for f in &self.files.clone() {
                        if let Some(resolved) = f.resolved.get(&name) {
                            match resolved {
                                Resolved::Function { name, cursor } => {
                                    println!("jumping to func {}", name);
                                    self.get_param_sources(&mut cursor.clone(), f);
                                    self.traverse_block(&mut cursor.clone(), f);
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

                                let vertex = Vertex::Source {
                                    tainting: t.clone(),
                                };
                                println!("pushing source");
                                self.graph.push(vertex, None, None);
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

    fn leave_node(&mut self, cursor: &mut TreeCursor, file: &File) {
        let node = cursor.node();
        //println!("kind: {}", node.kind());
        match node.kind() {
            "if_statement" | "do_statement" | "for_statement" => {
                self.context_stack.pop();
            }
            _ => (),
        }
    }

    fn get_param_sources(&mut self, cursor: &mut TreeCursor, file: &File) -> Vec<Taint<'a>> {
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
                            kind: "source".to_string(),
                            name: s,
                            scope: Scope {
                                global: true,
                                file: None,
                                class: None,
                                function: None,
                            },
                        };
                        taints.push(taint.clone());
                        self.taints.push(taint.clone());

                        /*
                        let vertex = Vertex::Source { tainting: taint };
                        self.graph.push(vertex, None, None);
                        */
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
                        let s: String = node_to_string(&cursor.node(), file.source_code);
                        let taint = Taint {
                            kind: "source".to_string(),
                            name: s,
                            scope: Scope {
                                global: true,
                                file: None,
                                class: None,
                                function: None,
                            },
                        };
                        taints.push(taint.clone());
                        self.taints.push(taint.clone());

                        /*
                        let vertex = Vertex::Source { tainting: taint };
                        self.graph.push(vertex, None, None);
                        */
                    }
            } else {
                visited = true;
            }
        }
        taints
    }

    fn find_name(&self, cursor: &mut TreeCursor, file: &File) -> Result<String, ()> {
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

    fn trace_taint(&mut self, cursor: &mut TreeCursor, file: &File, parent_taint: Taint<'a>) {
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
                        child_taint = Some(Taint {
                            kind: "variable".to_string(),
                            name,
                            scope: self.current_scope.clone(),
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
                        path.push(PathNode { name: name.clone() });
                        vertex = Some(Vertex::Unresolved {
                            parent_taint: parent_taint.clone(),
                            name,
                            path: path.clone(),
                        });
                    }
                }
                "method_call_expression" => {
                    let name = self.find_name(&mut cursor.clone(), &file);
                    if let Ok(name) = name {
                        path.push(PathNode { name: name.clone() });
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
}
