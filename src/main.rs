use crate::analyzer::*;
use crate::resolver::*;
use std::fs;
use tree_sitter::*;

pub mod analyzer;
pub mod graph;
pub mod resolver;
pub mod rules;

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
