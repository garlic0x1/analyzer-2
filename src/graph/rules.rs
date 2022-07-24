use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// a set of rules to alert for
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Rules {
    // sinks and their data
    vulns: HashMap<String, Vuln>,
    // sources just to get the analyzer started
    sources: HashSet<String>,
    hooks: HashSet<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vuln {
    pub sinks: HashMap<String, Option<Vec<u32>>>,
    pub sources: HashSet<String>,
    // funcs that make sink safe
    pub sanitizers: HashMap<String, Option<Vec<u32>>>,
    // funcs that make sink dangerous
    pub waypoints: Option<Vec<Waypoint>>,
}

impl Rules {
    pub fn from_yaml(filename: &str) -> Self {
        // parse yaml/json into our structure
        let contents = std::fs::read_to_string(filename).expect("no such file");
        let rules: Self = serde_yaml::from_str(&contents).expect("cant deserialize");
        rules
    }

    pub fn sources(&self) -> HashSet<String> {
        let mut outset = self.sources.clone();
        for (_kind, vuln) in self.vulns.iter() {
            outset.extend(vuln.sources.clone());
        }
        outset
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
