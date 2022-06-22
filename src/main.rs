use crate::resolver::*;
use daggy::Dag;
use std::fs;
use tree_sitter::*;

pub mod resolver;
pub mod rules;

// not same thing as context in last version
// this is to store hook/html stuff
#[derive(Debug)]
struct Context {
    kind: String,
    name: String,
}

// variable scope, same as old context, but with file
#[derive(Debug, Clone)]
struct Scope {
    global: bool,
    file: String,
    class: Option<String>,
    function: Option<String>,
}

#[derive(Debug)]
enum Vertex {
    Sink {
        // type of vuln (sqli, rce, etc)
        kind: String,
        // name to match
        name: String,
        // extra info
        code: String,
        position: Point,
        context: Context,
    },

    Assignment {
        // type of assingment (assign, append, return, pass, etc)
        kind: String,
        // taint to create
        tainting: Taint,
        // extra info
        code: String,
        position: Point,
        context: Context,
    },

    Sanitizer {
        // type of vuln (sqli, rce, etc)
        kind: String,
        // name to match
        name: String,
    },

    // passes through if unknown
    Unresolved,
    Break,
}

#[derive(Debug)]
struct Arc {
    // path of hooks, conditionals, and loops
    context_stack: Vec<Context>,
}

#[derive(Debug)]
enum Taint {
    Variable {
        // name of var
        name: String,
        scope: Scope,
        // allow us to connect to graph
        parent: Box<Vertex>,
    },
    Function {
        name: String,
        // allow us to connect to graph
        parent: Box<Vertex>,
    },
    // top of graph
    Source {
        name: String,
    },
    // these are the results (storing in this enum for graph)
    Sink {},
}

struct Vuln {}

struct Analyzer<'a> {
    files: Vec<resolver::File<'a>>,
    rules: rules::Rules,
    graph: Dag<Vertex, Arc>,
    context_stack: Vec<Context>,
    taints: Vec<Taint>,
    data_map: Vec<Vuln>,
}

impl<'a> Analyzer<'a> {
    pub fn new(files: Vec<File<'a>>, rules: rules::Rules) -> Self {
        let mut s = Self {
            files,
            rules,
            graph: Dag::new(),
            taints: Vec::new(),
            context_stack: Vec::new(),
            data_map: Vec::new(),
        };
        s.load_sources();
        return s;
    }

    pub fn traverse_block(&mut self, cursor: &mut TreeCursor, file: &File) {
        let mut visited = false;
        loop {
            if visited {
                if cursor.goto_next_sibling() {
                    let cont = self.enter_node(&mut cursor.clone(), &file);
                    if cont {
                        visited = true;
                        cursor.goto_parent();
                        continue;
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

    fn load_sources(&mut self) {
        for source in self.rules.sources.iter() {
            let taint = Taint::Source { name: source.name.clone() };
            self.taints.push(taint);
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
                                if name == &var_name {
                                    self.trace_taint(cursor);
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
    pub fn traverse(&mut self) -> Result<(), ()> {
        let file = self.files.get(0).expect("no files").clone(); // start with first (assumed main) file
        let t = file.tree.clone();
        let mut cursor = t.walk();
        let mut visited = false;
        let mut tabs = 0;
        loop {
            if visited {
                if cursor.goto_next_sibling() {
                    self.enter_node(&mut cursor.clone(), &file);
                    visited = false;
                } else if cursor.goto_parent() {
                    tabs -= 1;
                    //self.leave_node(&cursor.node(), &mut cursor.clone(), tabs)?;
                } else {
                    break;
                }
            } else if cursor.goto_first_child() {
                self.enter_node(&mut cursor.clone(), &file);
                tabs += 1;
            } else {
                visited = true;
            }
        }
        Ok(())
    }

    // more efficient that tracing up every name,
    // the idea is to do a depth first crawl of the branch
    // and identify the first name, will handle weird stuff
    // better than just getting the first child
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

    fn trace_taint(&mut self, cursor: &mut TreeCursor) {
        println!("tracing taint");
        while cursor.goto_parent() {
            match cursor.node().kind() {
                "assignment_expression" => {
                    // get name from variable_name.name or equivalent
                    // pass taints with unsanitized vuln categories, or none at all.
                }
                "function_call_expression" => {
                    // get name from child 0
                    // if sink break
                    // if sanitizer blacklist vuln
                }
                "method_call_expression" => {
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

fn main() -> Result<(), ()> {
    let ruleset = rules::Rules::new("");
    let source_code = fs::read_to_string("test.php").expect("failed to read file");
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_php::language())
        .expect("Error loading PHP parsing support");
    let tree: Tree = parser.parse(&source_code, None).unwrap();
    let mut file = File::new("filename".to_string(), &tree, &source_code);
    file.resolve();
    let mut files = Vec::new();
    files.push(file);

    let mut analyzer = Analyzer::new(files, ruleset);
    analyzer.traverse()?;
    Ok(())
}

fn node_to_string(node: &Node, source: &str) -> String {
    let slice = &source[node.start_byte()..node.end_byte()];
    slice.to_string()
}
