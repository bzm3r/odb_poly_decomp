use std::cell::Cell;

use crate::{geometry::Side, node::Node};

/// An edge from source to target.
#[derive(Clone, Debug)]
pub struct Edge<'a> {
    pub source: Cell<Option<&'a Node>>,
    pub target: Cell<Option<&Node>>,
    pub side: Side,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Containment {
    Strict, /* "contains_y": https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L105 */
    Weak, /* "inside_y": https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L112 */
}

impl<'a> Edge<'a> {
    pub fn new(source: &Node, target: &Node, side: Side) -> Self {
        Self {
            source: Cell::new(source.into()),
            target: Cell::new(target.into()),
            side,
        }
    }

    #[inline]
    pub fn src_x(&self) -> isize {
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
