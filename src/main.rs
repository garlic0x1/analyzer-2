use analyzer::analyzer::*;
use std::fs;
use tree::file::*;
use tree_sitter::*;
use utils::dumper::*;

pub mod analyzer;
pub mod graph;
pub mod tree;
pub mod utils;

fn main() {
    let source_code = fs::read_to_string("test.php").expect("failed to read file");

    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_php::language())
        .expect("Error loading PHP parsing support");
    let tree = parser.parse(&source_code, None).unwrap();
    let file = File::new("test.php".to_string(), &tree, &source_code);
    let dumper = Dumper::new(vec![&file]);
    println!("{}", dumper.dump());
    println!("{}", dumper.resolved());
    let mut analyzer =
        Analyzer::from_sources(vec![&file], vec!["_GET".to_string(), "_POST".to_string()]);
    println!("{}", analyzer.analyze());

    let source_code = fs::read_to_string("test0.php").expect("failed to read file");

    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_php::language())
        .expect("Error loading PHP parsing support");
    let tree = parser.parse(&source_code, None).unwrap();
    let file = File::new("test.php".to_string(), &tree, &source_code);
    let dumper = Dumper::new(vec![&file]);
    println!("{}", dumper.dump());
    println!("{}", dumper.resolved());
    let mut analyzer =
        Analyzer::from_sources(vec![&file], vec!["_GET".to_string(), "_POST".to_string()]);
    println!("{}", analyzer.analyze());
}
