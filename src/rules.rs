use std::collections::HashMap;

// a set of rules to alert for
pub struct Rules {
    pub sources: Vec<Source>,
}

pub struct Source {
    pub name: String,
    // sinks this source is dangerous in
    pub sinks: Vec<Sink>,
}

pub struct Sink {
    name: String,
    // specify which args are dangerous
    args: Vec<u32>,
    // funcs that make sink safe
    sanitizers: Vec<Sanitizer>,
    // funcs that make sink dangerous
    waypoints: Vec<Waypoint>,
}

struct Waypoint {
    name: String,
    // specify which args sanitize the function
    args: Vec<u32>,
}

struct Sanitizer {
    name: String,
    // specify which args sanitize the function
    args: Vec<u32>,
}

impl Rules {
    pub fn new(data_file: &str) -> Self {
        Self {
            sources: vec![Source {
                name: "_GET".to_string(),
                sinks: Vec::new(),
            }],
        }
    }
}
