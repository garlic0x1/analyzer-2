use crate::tree::cursor::*;

pub struct Traversal<'a> {
    cursor: Cursor<'a>,
    visited: bool,
    start: Cursor<'a>,
}

impl<'a> Traversal<'a> {
    pub fn new(cursor: Cursor<'a>) -> Self {
        Self {
            start: cursor.clone(),
            cursor,
            visited: false,
        }
    }
}

pub enum Order<'a> {
    Enter(Cursor<'a>),
    Leave(Cursor<'a>),
}

impl<'a> Iterator for Traversal<'a> {
    type Item = Order<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let last = self.cursor.clone();

        if self.visited {
            if self.cursor.goto_next_sibling() {
                self.visited = false;
                // returning enter
                return Some(Order::Leave(last));
            } else if self.cursor.goto_parent() {
                // returning leave
                return Some(Order::Leave(last));
            } else {
                return None;
            }
        } else {
            if self.cursor.goto_first_child() {
                // return enter
                return Some(Order::Enter(last));
            } else {
                // leave
                self.visited = true;
                return Some(Order::Enter(self.cursor.clone()));
            }
        }
    }
}
