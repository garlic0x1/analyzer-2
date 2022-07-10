use analyzer::analyzer::*;
use analyzer::taint::*;
//use graph::graph::*;
//use graph::rules::*;
use std::fs;
use tree::cursor::*;
use tree::file::*;
use tree::resolved::*;
use tree_sitter::*;
use utils::dumper::*;

//pub mod analyzer;
pub mod analyzer;
//pub mod graph;
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
    let mut analyzer =
        Analyzer::from_sources(vec![&file], vec!["_GET".to_string(), "_POST".to_string()]);
    analyzer.analyze();
}

/*
    //let curs = Cursor::new(tree.walk(), &file);
    //let resolved = curs.resolve();
    for (k, r) in resolved.iter() {
        println!("{:?}", r.parameters());
        println!("{}", k);
    }
let ruleset = rules::Rules::new("");
let source_code = fs::read_to_string("test.php").expect("failed to read file");
let source_code1 = fs::read_to_string("test1.php").expect("failed to read file");
let mut parser = Parser::new();
parser
    .set_language(tree_sitter_php::language())
    .expect("Error loading PHP parsing support");
let tree: Tree = parser.parse(&source_code, None).unwrap();
let tree1: Tree = parser.parse(&source_code1, None).unwrap();
let file1 = File::new("test1.php".to_string(), &tree1, &source_code1);
let file = File::new("test.php".to_string(), &tree, &source_code);
    let mut files = Vec::new();
    files.push(file.clone());
    files.push(file1);

    println!("done building files");
    //let mut analyzer = Analyzer::new(&files, ruleset);

fn node_to_string(node: &Node, source: &str) -> String {
    let slice = &source[node.start_byte()..node.end_byte()];
    slice.to_string()
}   //println!("{}", analyzer.graph.dump());
    */
