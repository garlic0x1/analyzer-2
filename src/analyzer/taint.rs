use std::collections::HashSet;

use crate::tree::cursor::*;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum TaintKind {
    Source,
    Global,
    Variable,
    Param,
    Return,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Taint {
    pub kind: TaintKind,
    pub name: String,
    pub scope: Scope,
}

impl Taint {
    pub fn new(cursor: Cursor, kind: TaintKind) -> Self {
        match kind {
            TaintKind::Return => Taint::new_variable(cursor),
            _ => Self {
                kind,
                name: cursor.name().expect("unnamed taint"),
                scope: Scope::new(cursor),
            },
        }
    }

    pub fn from_trace(cursor: Cursor) -> Result<Self, &str> {
        match cursor.kind() {
            "return_statement" => Ok(Taint::new_return(cursor)),
            "assignment_expression" => Ok(Taint::new_variable(cursor)),
            _ => Err("not a valid trace taint"),
        }
    }

    pub fn new_variable(cursor: Cursor) -> Self {
        Self {
            kind: TaintKind::Variable,
            name: cursor.name().expect("unnamed taint"),
            scope: Scope::new(cursor),
        }
    }

    pub fn new_source(name: String) -> Self {
        Self {
            kind: TaintKind::Source,
            name,
            scope: Scope::new_global(),
        }
    }

    pub fn new_return(cursor: Cursor) -> Self {
        Self {
            kind: TaintKind::Return,
            name: format!(
                "return {}",
                Scope::new(cursor.clone()).function.unwrap_or_default()
            ),
            scope: Scope::new_global(),
        }
    }

    pub fn new_param(cursor: Cursor) -> Self {
        Self {
            kind: TaintKind::Param,
            name: cursor.name().expect("unnamed taint"),
            scope: Scope::new(cursor),
        }
    }
}

// may need to change this to a hashmap for faster lookup times
#[derive(Debug)]
pub struct TaintList {
    list: HashSet<Taint>,
}

impl TaintList {
    pub fn new() -> Self {
        Self {
            list: HashSet::new(),
        }
    }

    pub fn push(&mut self, taint: Taint) {
        self.list.insert(taint);
    }

    pub fn remove(&mut self, taint: &Taint) {
        self.list.remove(taint);
    }

    pub fn get(&self, taint: &Taint) -> Option<Taint> {
        for t in self.list.iter() {
            if t.name == taint.name {
                return Some(t.clone());
            }
        }
        None
    }

    pub fn returns(&self) -> Vec<Taint> {
        let mut vec = Vec::new();
        for t in self.list.iter() {
            if t.kind == TaintKind::Return {
                vec.push(t.clone());
            }
        }
        vec
    }

    pub fn contains(&self, taint: &Taint) -> bool {
        for t in self.list.iter() {
            // dont exhaustively match global sources
            match t.kind {
                TaintKind::Global | TaintKind::Source | TaintKind::Return | TaintKind::Param => {
                    if t.name == taint.name {
                        return true;
                    }
                }
                TaintKind::Variable => {
                    if t.name == taint.name {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn clear_scope(&mut self, scope: &Scope) {
        self.list.retain(|sc| &sc.scope != scope);
    }

    pub fn clear_returns(&mut self) {
        self.list.retain(|sc| sc.kind != TaintKind::Return);
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Scope {
    pub filename: Option<String>,
    pub function: Option<String>,
    pub class: Option<String>,
}

impl Scope {
    pub fn new(cursor: Cursor) -> Self {
        let mut s = Self::new_global();
        s.filename = Some(cursor.filename());

        for cur in cursor.trace() {
            match cur.kind() {
                "method_declaration" | "function_definition" => {
                    s.function = cur.name();
                }
                _ => (),
            }
        }

        s
    }

    pub fn new_global() -> Self {
        Self {
            filename: None,
            function: None,
            class: None,
        }
    }

    pub fn contains(&self, other: &Self) -> bool {
        if let Some(_) = other.filename {
            if self.filename == other.filename {
                if self.class == other.class {
                    if self.function == other.function {
                        return true;
                    }
                }
            } else {
            }
        } else {
            return true;
        }
        //eprintln!("self: {:?} \nother: {:?}", self, other);
        false
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Context {
    pub kind: String,
    pub name: String,
}

impl Context {
    pub fn new(kind: String, name: String) -> Self {
        Self { kind, name }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct ContextStack {
    stack: Vec<Context>,
}

impl ContextStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// returns false if recursive
    pub fn push(&mut self, context: Context) -> bool {
        for ctx in self.stack.iter() {
            if ctx.eq(&context) {
                self.stack
                    .push(Context::new("".to_string(), "".to_string()));
                return false;
            }
        }
        self.stack.push(context);
        true
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
