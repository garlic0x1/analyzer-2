use analyzer::analyzer::*;
use graph::rules::*;
use tree::file::*;
use utils::dumper::*;

pub mod analyzer;
pub mod graph;
pub mod tree;
pub mod utils;

fn main() {
    let source_code = std::fs::read_to_string("test0.php").expect("failed to read file");
    let file = File::new("main".to_string(), source_code);

    // test dumper
    {
        let dumper = Dumper::new(vec![&file]);
        println!("{}", dumper.dump());
        println!("{}", dumper.resolved());
    }

    // create analyzer
    let mut analyzer =
        Analyzer::from_sources(vec![&file], vec!["_GET".to_string(), "_POST".to_string()]);
    // perform analysis
    analyzer.analyze();
    // get populated flow graph
    let graph = analyzer.graph();
    println!("{}", graph.dump());

    let rules = Rules::from_yaml("new.yaml");
    let paths = graph.walk();
    for path in paths {
        println!("{:?}", path);
        if rules.test_path(path) {
            println!("vuln");
        }
    }
}
