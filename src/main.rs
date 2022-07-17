use analyzer::analyzer::*;
use graph::rules::*;
use std::{io, io::prelude::*};
use tree::file::*;
use utils::dumper::Dumper;

pub mod analyzer;
pub mod graph;
pub mod tree;
pub mod utils;

fn main() {
    for line in io::stdin().lock().lines() {
        let mut files = Vec::new();
        for word in line.unwrap().split(' ') {
            if word.to_string().len() <= 1 {
                continue;
            }
            if word.contains("http") {
                let file = File::from_url(word).expect("failed to download");
                files.push(file);
                continue;
            }
            let file = File::new(word);
            files.push(file);
        }

        let file_refs: Vec<&File> = files.iter().map(|file| -> &File { &file }).collect();

        // create analyzer
        let mut analyzer =
            Analyzer::from_sources(file_refs, vec!["_GET".to_string(), "_POST".to_string()]);
        // perform analysis
        analyzer.analyze();
        // get populated flow graph
        let graph = analyzer.graph();
        println!("{}", graph.dump());

        println!("---");
        let rules = Rules::from_yaml("new.yaml");
        let paths = graph.walk();
        for path in paths.iter() {
            if rules.test_path(path) {
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
    }
}
