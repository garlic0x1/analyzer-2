use analyzer::analyzer::*;
use graph::rules::*;
use tree::file::*;
use utils::dumper::*;

pub mod analyzer;
pub mod graph;
pub mod tree;
pub mod utils;

fn main() {
    let file =
        File::from_url("https://plugins.svn.wordpress.org/wpw-newsletter/trunk/includes/model.php")
            .expect("failed to download file");

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
