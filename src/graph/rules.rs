use std::collections::HashMap;

// a set of rules to alert for
pub struct Rules {
    pub sinks: HashMap<String, Sink>,
}

pub struct Source {
    pub name: String,
}

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
    sources: Vec<Source>,
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
            sinks: HashMap::new(),
        }
    }

    fn from_yaml(&mut self, filename: &str) {
        // parse yaml/json into our structure
    }
}
