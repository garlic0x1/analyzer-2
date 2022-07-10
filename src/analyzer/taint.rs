use crate::tree::cursor::*;
use crate::tree::file::*;
use crate::tree::resolved::*;
use tree_sitter::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Taint {
    pub kind: String,
    pub name: String,
    pub scope: Scope,
}

impl Taint {
    pub fn new(cursor: Cursor) -> Self {
        Self {
            kind: cursor.kind().to_string(),
            name: cursor.name().expect("unnamed taint"),
            scope: Scope::new(cursor),
        }
    }
}

// may need to change this to a hashmap for faster lookup times
pub struct TaintList {
    vec: Vec<Taint>,
}

impl TaintList {
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }

    pub fn push(&mut self, taint: Taint) {
        self.vec.push(taint);
    }

    pub fn remove(&mut self, taint: &Taint) {
        let mut newvec = Vec::new();
        for t in self.vec.iter() {
            if t != taint {
                newvec.push(t.clone());
            }
        }
        self.vec = newvec;
    }

    pub fn contains(&self, taint: &Taint) -> bool {
        for t in self.vec.iter() {
            if t == taint {
                return true;
            }
        }
        false
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Scope {
    pub filename: Option<String>,
    pub function: Option<String>,
    pub class: Option<String>,
}

impl Scope {
    pub fn new(cursor: Cursor) -> Self {
        let mut s = Self::new_global();

        let mut closure = |cur: Cursor| -> bool {
            match cur.kind() {
                "function_definition" => {
                    s.function = cur.name();
                }
                _ => (),
            }
            true
        };

        let mut cur = cursor.clone();
        cur.trace(&mut closure);

        s
    }

    pub fn new_global() -> Self {
        Self {
            filename: None,
            function: None,
            class: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Context {
    pub kind: String,
    pub name: String,
}

impl Context {
    pub fn new(kind: String, name: String) -> Self {
        Self { kind, name }
    }
}

#[derive(Clone, Debug)]
pub struct ContextStack {
    stack: Vec<Context>,
}

impl ContextStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push(&mut self, context: Context) {
        self.stack.push(context);
    }

    pub fn pop(&mut self) -> Option<Context> {
        self.stack.pop()
    }

    pub fn contains(&self, other: &Self) -> bool {
        for i in 0..self.stack.len() {
            let s = self.stack.get(i).unwrap();
            let c = other.stack.get(i);
            if let Some(c) = c {
                if !(s.kind == c.kind && s.name == c.name) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}
