use daggy::Dag;
use super::*;

pub struct Graph {
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
        context: Context,
    },

    Resolved,
    Unresolved,
    Break,
}

pub struct Arc {
    // path of hooks, conditionals, and loops
    context_stack: Vec<Context>,
}

impl Graph {
    pub fn new() -> Self {
        Self {  } 
    }
}
