use crate::tree::cursor::*;
use crate::tree::file::*;
use crate::tree::resolved::*;
use tree_sitter::*;

#[derive(Clone, Debug)]
pub struct Taint<'a> {
    pub kind: &'a str,
    pub name: &'a str,
    pub scope: Scope,
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
