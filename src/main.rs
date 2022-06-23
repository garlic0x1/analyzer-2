use crate::resolver::*;
use crate::graph::*;
use daggy::Dag;
use std::fs;
use tree_sitter::*;

pub mod resolver;
pub mod rules;
pub mod graph;

// not same thing as context in last version
// this is to store hook/html stuff
#[derive(Clone, Debug)]
pub struct Context {
    kind: String,
    name: String,
}

// variable scope, same as old context, but with file
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Taint<'a> {
    Variable {
        // name of var
        name: String,
        scope: Scope<'a>,
    },
    Function {
        name: String,
        scope: Scope<'a>,
    },
    // top of graph
    Source {
        name: String,
    },
    // these are the results (storing in this enum for graph)
    Sink {},
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Scope<'a> {
    global: bool,
    file: Option<&'a File<'a>>,
    class: Option<String>,
    function: Option<String>,
}

pub struct Analyzer<'a> {
    files: Vec<resolver::File<'a>>,
    rules: rules::Rules,
    graph: Graph<'a>,
    context_stack: Vec<Context>,
    taints: Vec<Taint<'a>>,
    current_scope: Scope<'a>,
}

impl<'a> Analyzer<'a> {
    pub fn new(files: Vec<File<'a>>, rules: rules::Rules) -> Self {
        let mut s = Self {
            files: files.clone(),
            rules,
            current_scope: Scope{
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
            let taint = Taint::Source {
                name: source.name.clone(),
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
                    //self.leave_node(&cursor.node(), &mut cursor.clone(), tabs)?;
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
        match node.kind() {
            "function_definition" | "method_definition" | "class_definition" => return false,
            "function_call_expression" => return true,
            "method_call_expression" => return true,
            "variable_name" => {
                if let Ok(var_name) = self.find_name(&mut cursor.clone(), &file) {
                    println!("entering {}", var_name);
                    for t in self.taints.iter() {
                        match t {
                            Taint::Variable { name, .. } | Taint::Source { name, .. } => {
                                println!("matched {}", var_name);
                                if name.as_str() == var_name {
                                    self.trace_taint(cursor, &file, t.clone());
                                    return true;
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
            _ => return true,
        }
        return true;
    }

    fn find_name(&self, cursor: &mut TreeCursor, file: &File) -> Result<String, ()> {
        let mut visited = false;
        loop {
            if visited {
                if cursor.goto_next_sibling() {
                    visited = false;
                    if cursor.node().kind() == "name" {
                        let s: String = node_to_string(&cursor.node(), file.source_code);
                        println!("found name {}", s);
                        return Ok(s);
                    }
                } else if cursor.goto_parent() {
                } else {
                    break;
                }
            } else if cursor.goto_first_child() {
                if cursor.node().kind() == "name" {
                    let s: String = node_to_string(&cursor.node(), file.source_code);
                    println!("found name {}", s);
                    return Ok(s);
                }
            } else {
                visited = true;
            }
        }

        Err(())
    }

    fn trace_taint(&mut self, cursor: &mut TreeCursor, file: &File, parent_taint: Taint<'a>) {
        println!("tracing taint");
        while cursor.goto_parent() {
            match cursor.node().kind() {
                "assignment_expression" => {
                    if let Ok(name) = self.find_name(&mut cursor.clone(), &file){
                    let vertex = Vertex::Assignment {
                        kind: "assignment_expression".to_string(),
                        tainting: Taint::Variable { 
                            name,
                            scope: self.current_scope.clone(),
                        },
                    };
                    let arc = Arc {
                        context_stack: Vec::new(),
                    };
                    self.graph.push(vertex, arc, parent_taint.clone());
                    //println!("trace: [assignment] {:?}", name);
                    }
                    // get name from variable_name.name or equivalent
                    // pass taints with unsanitized vuln categories, or none at all.
                }
                "function_call_expression" => {
                    // get name from child 0
                    let name = self.find_name(&mut cursor.clone(), &file);
                    let vertex = Vertex::Unresolved {
                        name: name.unwrap(),
                    };
                    let arc = Arc {
                        context_stack: Vec::new(),
                    };
                    self.graph.push(vertex, arc, parent_taint.clone());
                    //println!("trace: {:?}", name);
                    // if sink break
                    // if sanitizer blacklist vuln
                }
                "method_call_expression" => {
                    let name = self.find_name(&mut cursor.clone(), &file);
                    println!("trace: {:?}", name);
                    // get name from child 0
                    // if sink break
                    // if sanitizer blacklist vuln
                }
                "echo_statement" => {
                    // sink
                    // break
                }
                _ => (),
            }
        }
    }
}

fn main() {
    let ruleset = rules::Rules::new("");
    let source_code = fs::read_to_string("test.php").expect("failed to read file");
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_php::language())
        .expect("Error loading PHP parsing support");
    let tree: Tree = parser.parse(&source_code, None).unwrap();
    let file = File::new("filename".to_string(), &tree, &source_code);
    let mut files = Vec::new();
    files.push(file);

    let mut analyzer = Analyzer::new(files, ruleset);
    analyzer.build_graph();
    println!("{}", analyzer.graph.dump());
}

fn node_to_string(node: &Node, source: &str) -> String {
    let slice = &source[node.start_byte()..node.end_byte()];
    slice.to_string()
}
