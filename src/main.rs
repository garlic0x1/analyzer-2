use crate::resolver::*;
use daggy::Dag;
use std::fs;
use tree_sitter::*;

pub mod resolver;

// not same thing as context in last version
// this is to store hook/html stuff
#[derive(Debug)]
struct Context {
    kind: String,
    name: String,
}

// variable scope, same as old context, but with file
#[derive(Debug)]
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
        // type of vuln (sqli, rce, etc)
        vulns: Vec<String>,
        // name of var
        name: String,
        scope: Scope,
        // allow us to connect to graph
        parent: Box<Vertex>,
    },
    Function {
        name: String,
        vulns: Vec<String>,
        // allow us to connect to graph
        parent: Box<Vertex>,
    },
    // top of graph
    Source {
        name: String,
        vulns: Vec<String>,
    },
    // these are the results (storing in this enum for graph)
    Sink {},
}

struct Vuln {}

struct Analyzer {
    files: Vec<resolver::File>,
    graph: Dag<Vertex, Arc>,
    context_stack: Vec<Context>,
    taints: Vec<Taint>,
    data_map: Vec<Vuln>,
}

impl Analyzer {
    pub fn new(files: Vec<File>) -> Self {
        Self {
            files,
            graph: Dag::new(),
            taints: Vec::new(),
            context_stack: Vec::new(),
            data_map: Vec::new(),
        }
    }

    /*
    pub fn resolve_names(&self, name: String) -> Result<(), QueryError> {
        let s = format!(
            "(function_definition
                            name: (name) @{})",
            name
        );
        let query = Query::new(tree_sitter_php::language(), s.as_str())?;
        let mut qcursor = QueryCursor::new();
        let tree_clone = self.tree.clone();
        let provider: tree_sitter::TextProvider = self.source_code.as_bytes();
        let matches = qcursor.matches(query, tree_clone.root_node(), provider);

        match query {
            Ok(query) => {
                let names = query.start_byte_for_pattern(0);
                println!("{:?}", names);
                Ok(())
            }
            Err(err) => {
                println!("{:?}", err);
                Err(())
            }
        }
    }
    */

    pub fn load_map() -> Result<(), ()> {
        Err(())
    }

    pub fn traverse(&mut self) -> Result<(), ()> {
        let file = self.files[0].clone();
        let t = file.tree.clone();
        let mut cursor = t.walk();
        let mut visited = false;
        let mut tabs = 0;
        loop {
            if visited {
                if cursor.goto_next_sibling() {
                    self.enter_node(&mut cursor.clone(), &file, tabs)?;
                    visited = false;
                } else if cursor.goto_parent() {
                    tabs -= 1;
                    //self.leave_node(&cursor.node(), &mut cursor.clone(), tabs)?;
                } else {
                    break;
                }
            } else if cursor.goto_first_child() {
                self.enter_node(&mut cursor.clone(), &file, tabs)?;
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
                        let s: String = node_to_string(&cursor.node(), file.source_code.as_str());
                        return Ok(s);
                    }
                } else if cursor.goto_parent() {
                } else {
                    break;
                }
            } else if cursor.goto_first_child() {
                if cursor.node().kind() == "name" {
                    let s: String = node_to_string(&cursor.node(), file.source_code.as_str());
                    return Ok(s);
                }
            } else {
                visited = true;
            }
        }

        Err(())
    }

    fn trace_taint(&mut self, cursor: &mut TreeCursor) {
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

    // call with a cloned TreeCursor to not lose our place in the traversal
    fn enter_node(&mut self, cursor: &mut TreeCursor, file: &File, tabs: usize) -> Result<(), ()> {
        let node = cursor.node();
        println!("{}", node.to_sexp());
        if node.is_named() && !node.is_extra() {
            println!("{}Kind: {}", "  ".repeat(tabs), node.kind());
            println!(
                "{}Code: {}",
                "  ".repeat(tabs),
                node_to_string(&node, file.source_code.as_str())
            );
        }
        match node.kind() {
            "function_call_expression" => println!(
                "{}Name: {:?}",
                "  ".repeat(tabs),
                self.find_name(&mut cursor.clone(), file)
            ),
            "method_call_expression" => println!(
                "{}Name: {:?}",
                "  ".repeat(tabs),
                self.find_name(&mut cursor.clone(), file)
            ),
            "variable_name" => println!(
                "{}Name: {:?}",
                "  ".repeat(tabs),
                self.find_name(&mut cursor.clone(), file)
            ),
            _ => (),
        }
        Ok(())
    }
}

fn main() -> Result<(), ()> {
    let source_code = fs::read_to_string("test.php").expect("failed to read file");
    let file = File::new("filename".to_string(), source_code);
    let mut files = Vec::new();
    files.push(file);

    let mut analyzer = Analyzer::new(files);
    analyzer.traverse()?;
    Ok(())
}

fn node_to_string(node: &Node, source: &str) -> String {
    let slice = &source[node.start_byte()..node.end_byte()];
    slice.to_string()
}
