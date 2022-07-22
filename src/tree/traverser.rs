use crate::tree::cursor::*;

pub struct Traversal<'a> {
    cursor: Cursor<'a>,
    start: Cursor<'a>,
    last: Option<Cursor<'a>>,
    concrete: bool,
    visited: bool,
    end: bool,
}

impl<'a> Traversal<'a> {
    /// abstract traversal (only named nodes)
    pub fn new(cursor: Cursor<'a>) -> Self {
        Self {
            start: cursor.clone(),
            last: None,
            cursor,
            visited: false,
            concrete: false,
            end: false,
        }
    }

    /// concrete traversal (all nodes)
    pub fn new_concrete(cursor: Cursor<'a>) -> Self {
        Self {
            start: cursor.clone(),
            last: None,
            cursor,
            visited: false,
            concrete: true,
            end: false,
        }
    }

    /// skip over this node
    pub fn pass(&mut self) {
        // if the one the user wants to skip isnt the first, go back to there, else end
        if let Some(cur) = &self.last {
            self.cursor = cur.clone();
        } else {
            self.end = true;
        }

        // switch to visited so we leave it
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

            // if visited go to next sibling or parent and leave visited node
            if self.cursor.goto_next_sibling() {
                // we havent visited this yet, break out of leave loop
                self.visited = false;
                if !self.concrete && !last.raw_cursor().node().is_named() {
                    return self.next();
                }
                return Some(Order::Leave(last));
            } else if self.cursor.goto_parent() {
                if !self.concrete && !last.raw_cursor().node().is_named() {
                    return self.next();
                }
                return Some(Order::Leave(last));
            } else {
                // break if we are at the root node
                self.end = true;
                if !self.concrete && !last.raw_cursor().node().is_named() {
                    return self.next();
                }
                return Some(Order::Leave(last));
            }
        } else {
            // if not visited, keep entering child nodes
            if self.cursor.goto_first_child() {
                if !self.concrete && !last.raw_cursor().node().is_named() {
                    return self.next();
                }
                return Some(Order::Enter(last));
            } else {
                // we are at a leaf, turn around
                self.visited = true;
                if !self.concrete && !self.cursor.raw_cursor().node().is_named() {
                    return self.next();
                }
                return Some(Order::Enter(self.cursor.clone()));
            }
        }
    }
}
