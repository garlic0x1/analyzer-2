use analyzer::analyzer::*;
use graph::rules::*;
use std::{io, io::prelude::*};
use tree::file::*;

pub mod analyzer;
pub mod graph;
pub mod tree;
pub mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rules = Rules::from_yaml("new.yaml")?;
    for line in io::stdin().lock().lines() {
        let mut files = Vec::new();
        for word in line.unwrap().split(' ') {
            if word.to_string().len() <= 1 {
                continue;
            }
            if word.contains("http") {
                eprintln!("downloading {}", word);
                if let Ok(file) = File::from_url(word) {
                    files.push(file);
                }
            } else {
                eprintln!("reading {}", word);
                let file = File::new(word);
                files.push(file);
            }
        }

        let file_refs: Vec<&File> = files.iter().map(|file| -> &File { &file }).collect();

        let dumper = crate::utils::dumper::Dumper::new(file_refs.clone());
        eprintln!("{}", dumper.dump());

        // create analyzer
        let mut analyzer = Analyzer::from_ruleset(file_refs, &rules);
        // perform analysis
        eprintln!("analyzing tree");
        analyzer.analyze();
        // get populated flow graph
        let graph = analyzer.graph();
        eprintln!("{}", graph.dump());

        let paths = graph.match_rules(&rules);
        println!("---");
        for path in paths.iter() {
            let filename = path.first().unwrap().filename();
            println!("file: {}", filename);
            println!("type: 'sqli'");
            println!("path:");
            for vert in path.iter() {
                println!("  - {}", vert.to_string());
            }
            println!("---");
        }
    }
    Ok(())
}
