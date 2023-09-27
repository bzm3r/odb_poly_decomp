use std::fmt::Debug;

use id_arena::Id;

use crate::{
    geometry::{GeometricId, Geometry, Side},
    node::{Node, NodeId},
};

pub type EdgeId = Id<Edge>;
impl GeometricId for EdgeId {
    type Item = Edge;
}

/// An edge from source to target.
#[derive(Clone, Copy)]
pub struct Edge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub side: Side,
}

impl Debug for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Edge[{}](src({})->tgt({}), {:?})",
            self.id.index(),
            self.source.index(),
            self.target.index(),
            self.side
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Containment {
    Strict, /* "contains_y": https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L105 */
    Weak, /* "inside_y": https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L112 */
}

impl Edge {
    pub fn new(id: EdgeId, source: NodeId, target: NodeId, side: Side) -> Self {
        Self {
            id,
            source,
            target,
            side,
        }
    }

    #[inline]
    pub fn src_x(&self, geometry: &Geometry) -> isize {
        self.source(geometry).x()
    }

    #[inline]
    pub fn src_y(&self, geometry: &Geometry) -> isize {
        self.source(geometry).y()
    }

    #[inline]
    pub fn tgt_y(&self, geometry: &Geometry) -> isize {
        self.target(geometry).y()
    }

    #[inline]
    pub fn min_max_y(&self, geometry: &Geometry) -> (isize, isize) {
        // TODO: put down where this comes from, and double check correctness
        let (src_y, tgt_y) = (self.src_y(geometry), self.tgt_y(geometry));
        (src_y.min(tgt_y), src_y.max(tgt_y))
    }

    #[inline]
    pub fn contains_y(&self, geometry: &Geometry, y: isize) -> bool {
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L105
        let (min_y, max_y) = self.min_max_y(geometry);
        (min_y <= y) && (y <= max_y)
    }

    #[inline]
    pub fn inside_y(&self, geometry: &Geometry, y: isize) -> bool {
        // Based on: https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L112
        let (min_y, max_y) = self.min_max_y(geometry);
        (min_y < y) && (y < max_y)
    }

    #[inline]
    pub fn set_source(&mut self, new: NodeId) {
        self.source = new;
    }

    #[inline]
    pub fn set_target(&mut self, new: NodeId) {
        self.target = new;
    }

    #[inline]
    pub fn source(&self, geometry: &Geometry) -> Node {
        geometry[self.source]
    }

    #[inline]
    pub fn target(&self, geometry: &Geometry) -> Node {
        geometry[self.target]
    }
}
