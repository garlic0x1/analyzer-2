use crate::cursor::*;

pub enum Resolved<'a> {
    Function { name: String, cursor: Cursor<'a> },
}

impl<'a> Resolved<'a> {
    pub fn new_function(name: String, cursor: Cursor<'a>) -> Self {
        Self::Function { name, cursor }
    }

    /// returns vec of resolved parameter names
    /// empty if not function variant
    pub fn parameters(&self) -> Vec<String> {
        let mut v = Vec::new();

        match self {
            Resolved::Function { name, cursor } => {
                // create mutable closure
                let mut enter_node = |cur: &Cursor| -> bool {
                    if cur.kind() == "simple_parameter" {
                        if let Some(n) = cur.name() {
                            v.push(n);
                        }
                        return false;
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