use super::cursor::*;

pub enum Resolved<'a> {
    Function { cursor: Cursor<'a> },
    Root { cursor: Cursor<'a> },
}

impl<'a> Resolved<'a> {
    pub fn new_function(cursor: Cursor<'a>) -> Self {
        Self::Function { cursor }
    }

    pub fn new_root(cursor: Cursor<'a>) -> Self {
        Self::Root { cursor }
    }

    /// returns vec of resolved parameter names
    /// empty if not function variant
    pub fn parameters(&self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();

        match self {
            Resolved::Function { cursor } => {
                // create mutable closure
                let mut enter_node = |cur: Cursor<'a>| -> bool {
                    if cur.kind() == "simple_parameter" {
                        if let Some(n) = cur.name() {
                            v.push(n.to_string());
                        }
                        return true;
                    }
                    true
                };

                // traverse with closure
                let mut cursor = cursor.clone();
                cursor.traverse(&mut enter_node, &mut |_| ());

                v
            }
            // return empty if not function
            _ => v,
        }
    }
}
