use crate::tree::file::*;
use std::collections::HashMap;

pub struct Repository {
    name: String,
    version: String,
    files: HashMap<String, File>,
}

impl Repository {
    pub fn from_url(url: &str, version: &str) -> Self {
        let mut s = Self {
            name: url.to_string(),
            version: version.to_string(),
            files: HashMap::new(),
        };

        s
    }
}
