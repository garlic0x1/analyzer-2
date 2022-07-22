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

    // pub fn test_verts(&self, verts: &Vec<Vertex>) {
    //     let mut path = Vec::new();
    //     let mut last_vert = Option<Vertex<'a>>;
    //     let mut first = true;
    //     for vert in verts.iter() {
    //         if first {
    //             let sink_name = vert.paths.first().expect("empty path").name().unwrap();
    //         }
    //         path.extend();
    //         first = false;
    //     }
    // }

    pub fn test_path(&self, path: &Vec<Cursor>) -> bool {
        let sink_name = &path.first().expect("empty path").name().unwrap();
        let &sink_kind = &path.first().expect("empty").kind();
        let mut sink: Option<Sink> = None;
        if let Some(nsink) = self.sinks.get(sink_name) {
            sink = Some(nsink.clone());
        }
        if let Some(nsink) = self.sinks.get(sink_kind) {
            sink = Some(nsink.clone());
        }
        if let Some(sink) = sink {
            for c in path.iter() {
                if let Some(pname) = c.name() {
                    if sink.sanitizers.contains_key(&pname)
                        || sink.sanitizers.contains_key(c.kind())
                    {
                        return false;
                    }
                }
            }

            return true;
        }
        false
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Waypoint {
    // specify which args sanitize the function
    args: Option<Vec<u32>>,
}
