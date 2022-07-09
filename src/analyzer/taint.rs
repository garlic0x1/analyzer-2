use crate::tree::cursor::*;
use crate::tree::file::*;
use crate::tree::resolved::*;
use tree_sitter::*;

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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
    kind: String,
    name: String,
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
