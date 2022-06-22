use daggy::Dag;
use std::collections::HashMap;
use super::*;

pub struct Graph<'a> {
    dag: Dag<Vertex<'a>, Arc>,
    leaves: HashMap<&'a Taint<'a>, &'a Vertex<'a>>,
}

pub enum Vertex<'a> {
    Assignment {
        // type of assingment (assign, append, return, pass, etc)
        kind: String,
        // taint to create
        tainting: Taint<'a>,
        // extra info
        code: String,
        position: Point,
        context_stack: Vec<Context>,
    },

    Resolved,
    Unresolved,
    Break,
}

pub struct Arc {
    // path of hooks, conditionals, and loops
    context_stack: Vec<Context>,
}

impl<'a> Graph<'a> {
    pub fn new() -> Self {
        Self { 
            dag: Dag::new(),
            leaves: HashMap::new(),
        } 
    }

    pub fn push(&mut self, parent_taint: &Taint) {
        let leaf = self.leaves.get(parent_taint);
    }
}
