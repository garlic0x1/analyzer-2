use tree_sitter::*;

pub struct Cursor<'a> {
    cursor: TreeCursor<'a>,
}
