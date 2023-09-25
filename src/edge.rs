use crate::{node::Node, scanner::Side};

/// An edge from source to target.
#[derive(Clone, Copy, Debug)]
pub struct Edge<'a> {
    pub source: &'a Node<'a>,
    pub target: &'a Node<'a>,
    pub side: Side,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Containment {
    Strict, /* "contains_y": https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L105 */
    Weak, /* "inside_y": https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L112 */
}

impl<'a> Edge<'a> {
    pub fn new(source: &'a Node<'a>, target: &'a Node<'a>, side: Side) -> Self {
        Self {
            source,
            target,
            side,
        }
    }

    #[inline]
    pub fn src_x(&self) -> isize {
        self.source.x()
    }

    #[inline]
    pub fn src_y(&self) -> isize {
        self.source.y()
    }

    #[inline]
    pub fn tgt_y(&self) -> isize {
        self.target.y()
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
}
