use super::cursor::*;

#[derive(Clone)]
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

    pub fn cursor(&self) -> Cursor {
        match self {
            Resolved::Function { cursor } => cursor.clone(),
            Resolved::Root { cursor } => cursor.clone(),
        }
    }

    /// returns vec of resolved parameter names
    /// empty if not function variant
    pub fn parameters(&self) -> Vec<Cursor<'a>> {
        let mut v = Vec::new();

        match self {
            Resolved::Function { cursor } => {
                // create mutable closure
                let mut enter_node = |cur: Cursor<'a>| -> Breaker {
                    if cur.kind() == "simple_parameter" {
                        v.push(cur);
                    }
                    Breaker::Continue
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

    pub fn dump_parameters(&self) -> String {
        let mut string = String::new();
        for cur in self.parameters().iter() {
            string.push_str(&cur.name().unwrap());
            string.push(' ');
        }
        string
    }

    pub fn name(&self) -> String {
        self.cursor().name().unwrap()
    }
}
