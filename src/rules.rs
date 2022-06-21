use std::collections::HashMap;

// a set of rules to alert for
pub struct Rules {
    sources: Vec<Source>,
}

struct Source {
    name: String,
    // sinks this source is dangerous in
    sinks: Vec<Sink>,
}

struct Sink {
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
