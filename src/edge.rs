use crate::{node::Node, scanner::Side};

/// An edge from source to target.
#[derive(Clone, Copy, Debug)]
pub struct Edge<'a> {
    source: &'a Node<'a>,
    target: &'a Node<'a>,
    side: Side,
}

impl<'a> Edge<'a> {
    pub fn new(source: &'a Node<'a>, target: &'a Node<'a>, side: Side) -> Self { Self { source, target, side } }

    pub fn src_x(&self) -> isize {
        self.source.x()
    }
}
