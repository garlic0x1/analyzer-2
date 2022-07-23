use crate::tree::cursor::*;

pub struct Trace<'a> {
    cursor: Cursor<'a>,
    concrete: bool,
}

impl<'a> Trace<'a> {
    pub fn new(cursor: Cursor<'a>) -> Self {
        Self {
            cursor,
            concrete: false,
        }
    }
}

impl<'s> Iterator for Trace<'s> {
    type Item = Cursor<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.goto_parent() {
            return Some(self.cursor.clone());
        } else {
            return None;
        }
    }
}
