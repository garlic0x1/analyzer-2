use std::fs;
use tree_sitter::{Node, Parser, Tree, TreeCursor};
use tree_sitter_php;

// not same thing as context in last version
// this is to store hook/html stuff
struct Context {
    kind: String,
    name: String,
}

// variable scope, same as old context, but with file
struct Scope {
    file: String,
    class: Option<String>,
    function: Option<String>,
}

// there should be major and minor vertices
// major vertices are parts of code where an
// assignment or function call happens. A minor
// vertex is an object on the stack to be
// traced through
enum Vertex {
    Major {
        kind: String,

        // help sonic find the code
        source_code: String,
        position: tree_sitter::Point,
        context: Context,

        // these are the children on the graph
        // (a child may have more than one parent)
        arcs: Vec<Arc>,
    },

    Minor {
        kind: String,
        name: String,
    },
}

struct Arc {
    stack: Vec<Vertex>,
    // an immutable pointer to a vertex
    pointer: *const Vertex,
}

struct Taint {
    kind: String,
    name: String,
    scope: Scope,
}

struct Analyzer {
    tree: Tree,
    source_code: String,
    context_stack: Vec<Context>,
    taints: Vec<Taint>,
    files: Vec<String>,
}

impl Analyzer {
    pub fn new(tree: Tree, source_code: String) -> Self {
        Self {
            tree,
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
