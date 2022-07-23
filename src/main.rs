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
                eprintln!("downloading {}", word);
                let file = File::from_url(word).expect("failed to download");
                files.push(file);
                continue;
            }
            let file = File::new(word);
            files.push(file);
        }

        let file_refs: Vec<&File> = files.iter().map(|file| -> &File { &file }).collect();

        let dumper = crate::utils::dumper::Dumper::new(file_refs.clone());
        println!("{}", dumper.dump());
        let mut cur = files.first().unwrap().cursor();
        cur.goto_first_child();
        cur.goto_next_sibling();
        println!("{}", Dumper::dump_cursor(cur));

        // create analyzer
        let mut analyzer =
            Analyzer::from_sources(file_refs, vec!["_GET".to_string(), "_POST".to_string()]);
        // perform analysis
        eprintln!("analyzing");
        analyzer.analyze();
        // get populated flow graph
        let graph = analyzer.graph();
        eprintln!("{}", graph.dump());

        let rules = Rules::from_yaml("new.yaml");
        eprintln!("routing");
        let paths = graph.walk_verts();
        println!("{:?}", paths);
        println!("---");
        for path in paths.iter() {
            if rules.test_path(&graph.verts_to_path(path.clone())) {
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
