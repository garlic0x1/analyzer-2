use crate::tree::cursor::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// a set of rules to alert for

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Rules {
    // sinks and their data
    pub sinks: HashMap<String, Sink>,
    // sources just to get the analyzer started
    pub sources: HashSet<String>,
    // hooks to treat like function calls along with
    // arg index of the hooked function
    pub hooks: HashSet<String>,
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

    pub fn test_path(&self, path: Vec<Cursor>) -> bool {
        let sink_name = path.first().expect("empty path").name().unwrap();
        if let Some(sink) = self.sinks.get(&sink_name) {
            for c in path.iter() {
                if let Some(pname) = c.name() {
                    if sink.sanitizers.contains_key(&pname) {
                        return false;
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
    name: String,
    // specify which args are dangerous
    // all are dangerous if None
    args: Option<Vec<u32>>,
    // funcs that make sink safe
    sanitizers: HashMap<String, Sanitizer>,
    // funcs that make sink dangerous
    waypoints: Vec<Waypoint>,
    // sources that make the sink vuln
    sources: Vec<String>,
}

impl Sink {
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Waypoint {
    name: String,
    // specify which args sanitize the function
    args: Vec<u32>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Sanitizer {
    name: String,
    // specify which args sanitize the function
    args: Vec<u32>,
}
