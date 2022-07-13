use crate::graph::flowgraph::*;
use crate::tree::cursor::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// a set of rules to alert for

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Rules {
    // sinks and their data
    sinks: HashMap<String, Sink>,
    // sources just to get the analyzer started
    sources: HashSet<String>,
    hooks: HashSet<String>,
}

impl Rules {
    pub fn new(
        sinks: HashMap<String, Sink>,
        sources: HashSet<String>,
        hooks: HashSet<String>,
    ) -> Self {
        Self {
            sinks,
            sources,
            hooks,
        }
    }

    pub fn from_yaml(filename: &str) -> Self {
        // parse yaml/json into our structure
        let contents = std::fs::read_to_string(filename).expect("no such file");
        serde_yaml::from_str(&contents).expect("cant deserialize")
    }

    pub fn sources(&self) -> &HashSet<String> {
        &self.sources
    }

    pub fn sinks(&self) -> &HashMap<String, Sink> {
        &self.sinks
    }

    pub fn hooks(&self) -> &HashSet<String> {
        &self.hooks
    }

    pub fn test_path(&self, path: Vec<Vertex>) -> bool {
        let sink_name = path
            .clone()
            .first()
            .expect("empty path")
            .path
            .first()
            .unwrap()
            .name()
            .unwrap();

        if let Some(sink) = self.sinks.get(&sink_name) {
            for segment in path.iter() {
                for c in segment.path.iter() {
                    if let Some(pname) = c.name() {
                        if sink.sanitizers.contains_key(&pname) {
                            return false;
                        }
                    }
                }
            }

            return true;
        }
        false
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Sink {
    // specify which args are dangerous
    // all are dangerous if None
    args: Option<Vec<u32>>,
    // funcs that make sink safe
    sanitizers: HashMap<String, Option<Vec<u32>>>,
    // funcs that make sink dangerous
    waypoints: Option<Vec<Waypoint>>,
    // sources that make the sink vuln
    sources: Vec<String>,
}

impl Sink {
    // pub fn name(&self) -> &str {
    //     &self.name.unwrap_or_default()
    // }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Waypoint {
    // specify which args sanitize the function
    args: Option<Vec<u32>>,
}
