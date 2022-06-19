use std::fs;
use tree_sitter::{Node, Parser, Tree, TreeCursor, Point};
use daggy::Dag;
use tree_sitter_php;

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

    Break 
}

#[derive(Debug)]
struct Arc {
    stack: Vec<Context>,
}

#[derive(Debug)]
struct Taint {
    // type of vuln (sqli, rce, etc)
    vuln: String,
    // name of var
    name: String,
    scope: Scope,
}

struct Analyzer {
    tree: Tree,
    graph: Dag<Vertex, Arc>,
    source_code: String,
    context_stack: Vec<Context>,
    taints: Vec<Taint>,
    files: Vec<String>,
}

impl Analyzer {
    pub fn new(tree: Tree, source_code: String) -> Self {
        Self {
            tree,
            graph: Dag::new(),
            source_code,
            taints: Vec::new(),
            context_stack: Vec::new(),
            files: Vec::new(),
        }
    }

    pub fn traverse(&mut self) -> Result<(), ()> {
        let t = self.tree.clone();
        let mut cursor = t.walk();
        let mut visited = false;
        let mut tabs = 0;
        loop {
            if visited {
                if cursor.goto_next_sibling() {
                    self.enter_node(&mut cursor.clone(), tabs)?;
                    visited = false;
                } else if cursor.goto_parent() {
                    tabs -= 1;
                    //self.leave_node(&cursor.node(), &mut cursor.clone(), tabs)?;
                } else {
                    break;
                }
            } else if cursor.goto_first_child() {
                self.enter_node(&mut cursor.clone(), tabs)?;
                tabs += 1;
            } else {
                visited = true;
            }
        }
        Ok(())
    }

    fn trace_taint(&mut self, cursor: &mut TreeCursor) {
        while cursor.goto_parent() {
            match cursor.node().kind() {
                "function_call_expression" => {}
                "method_call_expression" => {}
                "echo_statement" => {}
                _ => (),
            }
        }
    }

    fn trace_name(&self, cursor: &mut TreeCursor) {
        while cursor.goto_parent() {
            match cursor.node().kind() {
                "function_call_expression" => {
                    println!("FUNC CALL NAME");
                    break;
                }
                "variable_name" => {
                    println!("VAR NAME");
                    break;
                }
                _ => (),
            }
        }
    }

    // call with a cloned TreeCursor to not lose our place in the traversal
    fn enter_node(&mut self, cursor: &mut TreeCursor, tabs: usize) -> Result<(), ()> {
        let node = cursor.node();
        if node.is_named() && !node.is_extra() {
            println!("{}Kind: {}", "  ".repeat(tabs), node.kind());
            println!(
                "{}Code: {}",
                "  ".repeat(tabs),
                node_to_string(&node, self.source_code.as_str())
            );
        }
        match node.kind() {
            "name" => self.trace_name(cursor),
            _ => (),
        }
        Ok(())
    }
}

fn main() -> Result<(), ()> {
    let source_code = fs::read_to_string("test.php").expect("failed to read file");
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_php::language())
        .expect("Error loading PHP parsing support");
    let tree = parser.parse(source_code.clone(), None).unwrap();

    let mut analyzer = Analyzer::new(tree.clone(), source_code);
    analyzer.traverse()?;
    Ok(())
}

fn node_to_string(node: &Node, source: &str) -> String {
    let slice = &source[node.start_byte()..node.end_byte()];
    slice.to_string()
}
