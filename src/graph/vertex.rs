use crate::analyzer::taint::*;
use crate::tree::cursor::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Vertex<'a> {
    assign: Option<Taint>,
    context: ContextStack,
    parents: HashMap<Cursor<'a>, PathItem<'a>>,
    global_sources: HashMap<Cursor<'a>, PathItem<'a>>,
}

impl<'a> Vertex<'a> {
    pub fn new(assign: Option<Taint>, context: ContextStack) -> Self {
        Self {
            assign,
            context,
            parents: HashMap::new(),
            global_sources: HashMap::new(),
        }
    }

    pub fn parents(&self) -> &HashMap<Cursor<'a>, PathItem<'a>> {
        &self.parents
    }

    pub fn add_parent(&mut self, parent: Cursor<'a>, path: PathItem<'a>) {
        self.parents.insert(parent, path);
    }

    pub fn sources(&self) -> &HashMap<Cursor<'a>, PathItem<'a>> {
        &self.global_sources
    }

    pub fn add_source(&mut self, source: Cursor<'a>, path: PathItem<'a>) {
        self.global_sources.insert(source, path);
    }

    pub fn context(&self) -> &ContextStack {
        &self.context
    }

    pub fn assign(&self) -> &Option<Taint> {
        &self.assign
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct PathItem<'a> {
    source: Taint,
    path: Vec<Cursor<'a>>,
}

impl<'a> PathItem<'a> {
    pub fn new(source: Taint, path: Vec<Cursor<'a>>) -> Self {
        Self { source, path }
    }

    pub fn contains(&self, cursor: &'a Cursor<'a>) -> bool {
        self.path.contains(cursor)
    }

    pub fn segments(&self) -> std::slice::Iter<'_, Cursor<'a>> {
        self.path.iter()
    }

    pub fn path_vec(&self) -> &Vec<Cursor<'a>> {
        &self.path
    }

    pub fn source(&self) -> &Taint {
        &self.source
    }

    pub fn to_str(&self) -> &str {
        match self.path.first() {
            Some(cur) => cur.to_str(),
            None => "",
        }
    }
}
