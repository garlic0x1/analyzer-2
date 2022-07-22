use crate::tree::cursor::*;

pub struct Traversal<'a> {
    cursor: Cursor<'a>,
    last: Option<Cursor<'a>>,
    start: Cursor<'a>,
    visited: bool,
    end: bool,
}

impl<'a> Traversal<'a> {
    pub fn new(cursor: Cursor<'a>) -> Self {
        Self {
            start: cursor.clone(),
            last: None,
            cursor,
            visited: false,
            end: false,
        }
    }

    pub fn pass(&mut self) {
        if let Some(cur) = &self.last {
            self.cursor = cur.clone();
        } else {
            self.end = true;
        }
        self.visited = true;
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
        self.last = Some(last.clone());

        if self.end {
            return None;
        }

        if self.visited {
            // break when we have completely visited start
            if last == self.start {
                self.end = true;
            }

            if self.cursor.goto_next_sibling() {
                self.visited = false;
                // leave
                return Some(Order::Leave(last));
            } else if self.cursor.goto_parent() {
                // leave
                return Some(Order::Leave(last));
            } else {
                self.end = true;
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
