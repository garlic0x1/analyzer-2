use crate::tree::cursor::*;

pub struct Traversal<'a> {
    cursor: Cursor<'a>,
    visited: bool,
    start: Cursor<'a>,
    last: bool,
}

impl<'a> Traversal<'a> {
    pub fn new(cursor: Cursor<'a>) -> Self {
        Self {
            start: cursor.clone(),
            cursor,
            visited: false,
            last: false,
        }
    }
}

pub enum Order<'a> {
    Enter(Cursor<'a>),
    Leave(Cursor<'a>),
}

/// preorder and postorder together, for pushing and popping to context stack
impl<'a> Iterator for Traversal<'a> {
    type Item = Order<'a>;

    /// get the next step in iteration
    fn next(&mut self) -> Option<Self::Item> {
        let last = self.cursor.clone();

        if self.last {
            return None;
        }

        if self.visited {
            // break when we have completely visited start
            if last == self.start {
                self.last = true;
            }

            if self.cursor.goto_next_sibling() {
                self.visited = false;
                // leave
                return Some(Order::Leave(last));
            } else if self.cursor.goto_parent() {
                // leave
                return Some(Order::Leave(last));
            } else {
                self.last = true;
                return Some(Order::Leave(last));
            }
        } else {
            if self.cursor.goto_first_child() {
                // enter
                return Some(Order::Enter(last));
            } else {
                // enter
                self.visited = true;
                return Some(Order::Enter(self.cursor.clone()));
            }
        }
    }
}
