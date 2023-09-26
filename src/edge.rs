use std::cell::Cell;

use crate::{index::{NodeIdx, EdgeIdx}, node::{Node, Nodes}, scanner::Side};

#[derive(Clone, Debug, Default)]
pub struct Edges<'a>(Vec<Edge<'a>>);

impl<'a> Edges<'a> {
    pub fn get(&self, idx: &EdgeIdx) -> Option<&'a Edge> {
        let ix: Option<usize> = idx.into();
        ix.and_then(|ix| self.0.get(ix))
    }
}

/// An edge from source to target.
#[derive(Clone, Debug)]
pub struct Edge<'a> {
    pub node_vec: &'a Nodes<'a>,
    pub source: NodeIdx,
    pub target: NodeIdx,
    pub side: Side,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Containment {
    Strict, /* "contains_y": https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L105 */
    Weak, /* "inside_y": https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L112 */
}

impl<'a> Edge<'a> {
    pub fn new(
        node_vec: &'a Vec<Node<'a>>,
        source: NodeIdx,
        target: NodeIdx,
        side: Side,
    ) -> Self {
        Self {
            source: Cell::new(source.into()),
            target: Cell::new(target.into()),
            side,
            node_vec,
        }
    }

    #[inline]
    pub fn src_x(&self) -> isize {
        self.Vec<Node<'a>>
        self.source.get().unwrap().x()
    }

    #[inline]
    pub fn src_y(&self) -> isize {
        self.source.get().unwrap().y()
    }

    #[inline]
    pub fn tgt_y(&self) -> isize {
        self.target.get().unwrap().y()
    }

    #[inline]
    pub fn min_max_y(&self) -> (isize, isize) {
        (
            self.src_y().min(self.tgt_y()),
            self.src_y().max(self.tgt_y()),
        )
    }

    // Based on:
    // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L105
    #[inline]
    pub fn contains_y(&self, y: isize) -> bool {
        let (min_y, max_y) = self.min_max_y();
        (min_y <= y) && (y <= max_y)
    }

    // Based on: https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L112
    #[inline]
    pub fn inside_y(&self, y: isize) -> bool {
        let (min_y, max_y) = self.min_max_y();
        (min_y < y) && (y < max_y)
    }

    #[inline]
    pub fn set_source(&self, new: &Node) {
        self.source.replace(new.into());
    }

    #[inline]
    pub fn set_target(&self, new: &Node) {
        self.target.replace(new.into());
    }

    #[inline]
    pub fn source(&self) -> Option<&Node> {
        self.source.get()
    }

    #[inline]
    pub fn target(&self) -> Option<&Node> {
        self.target.get()
    }
}
