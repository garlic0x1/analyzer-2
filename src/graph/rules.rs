use crate::tree::cursor::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vuln {
    sinks: HashMap<String, Option<Vec<u32>>>,
    sources: HashSet<String>,
    // funcs that make sink safe
    sanitizers: HashMap<String, Option<Vec<u32>>>,
    // funcs that make sink dangerous
    waypoints: Option<Vec<Waypoint>>,
}

pub enum VertKind {
    Source,
    Sanitizer,
    Sink,
}

impl Vuln {
    /// standardize cursor to matching string
    pub fn identify(&self, cursor: Cursor) -> Option<VertKind> {
        let kind = cursor.kind().to_string();
        let name = match cursor.kind() {
            "cast_type" => {
                let s = format!("({})", cursor.to_str());
                println!("cast type {s}");
                s
            }
            _ => cursor.name().unwrap_or_default(),
        };

        if self.has_sanitizer(&kind) || self.has_sanitizer(&name) {
            return Some(VertKind::Sanitizer);
        }

        if self.has_source(&kind) || self.has_source(&name) {
            return Some(VertKind::Source);
        }

        if self.has_sink(&kind) || self.has_sink(&name) {
            return Some(VertKind::Sink);
        }

        None
    }

    pub fn sources(&self) -> &HashSet<String> {
        &self.sources
    }

    pub fn has_source(&self, source: &String) -> bool {
        self.sources.contains(source)
    }

    pub fn sinks(&self) -> &HashMap<String, Option<Vec<u32>>> {
        &self.sinks
    }

    pub fn has_sink(&self, sink: &String) -> bool {
        self.sinks.contains_key(sink)
    }

    pub fn sanitizers(&self) -> &HashMap<String, Option<Vec<u32>>> {
        &self.sanitizers
    }

    pub fn has_sanitizer(&self, sanitizer: &String) -> bool {
        self.sanitizers.contains_key(sanitizer)
    }
}

// a set of rules to alert for
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Rules {
    // sinks and their data
    vulns: HashMap<String, Vuln>,
    // sources just to get the analyzer started
    sources: HashSet<String>,
    hooks: HashSet<String>,
}

impl Rules {
    pub fn from_yaml(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // parse yaml/json into our structure
        let contents = std::fs::read_to_string(filename)?;
        let rules: Self = serde_yaml::from_str(&contents)?;
        Ok(rules)
    }

    pub fn sources(&self) -> HashSet<String> {
        let mut outset = self.sources.clone();
        for (_kind, vuln) in self.vulns.iter() {
            outset.extend(vuln.sources.clone());
        }
        outset
    }

    pub fn has_source(&self, source: &String) -> bool {
        self.sources().contains(source)
    }

    pub fn sinks(&self) -> HashMap<String, Option<Vec<u32>>> {
        let mut names = HashMap::new();
        for (_kind, vuln) in self.vulns.iter() {
            for (name, sink) in vuln.sinks.iter() {
                names.insert(name.clone(), sink.clone());
            }
        }

        names
    }

    pub fn hooks(&self) -> &HashSet<String> {
        &self.hooks
    }

    pub fn vulns(&self) -> &HashMap<String, Vuln> {
        &self.vulns
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct Waypoint {
    // specify which args sanitize the function
    args: Option<Vec<u32>>,
}
