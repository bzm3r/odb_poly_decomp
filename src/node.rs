use id_arena::Id;
use std::cmp::Ordering;
use std::fmt::Debug;

use crate::{
    edge::EdgeId,
    geometry::{GeometricId, Side},
    point::Point,
};

pub type NodeId = Id<Node>;
impl GeometricId for NodeId {
    type Item = Node;
}

#[derive(Clone, Copy)]
pub struct Node {
    pub id: NodeId,
    pub point: Point,
    inc_edge: Option<EdgeId>,
    out_edge: Option<EdgeId>,
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node[{}](inc({})->{:?}->out({}))",
            self.id.index(),
            match self.inc_edge {
                Some(id) => format!("Some({})", id.index()),
                None => "None".to_string(),
            },
            self.point,
            match self.out_edge {
                Some(id) => format!("Some({})", id.index()),
                None => "None".to_string(),
            }
        )
    }
}

impl Node {
    pub fn new(
        point: Point,
        id: NodeId,
        inc_edge: Option<EdgeId>,
        out_edge: Option<EdgeId>,
    ) -> Self {
        Self {
            id,
            point,
            inc_edge,
            out_edge,
        }
    }

    #[inline]
    pub fn set_inc_edge(&mut self, inc: EdgeId) {
        debug_assert!(self.inc_edge.is_none());
        self.inc_edge.replace(inc);
    }

    #[inline]
    pub fn set_out_edge(&mut self, out: EdgeId) {
        debug_assert!(self.out_edge.is_none());
        self.out_edge.replace(out);
    }

    #[inline]
    pub fn which_side(&self, other: &Node) -> Option<Side> {
        self.point.which_side(&other.point)
    }

    #[inline]
    pub fn x(&self) -> isize {
        self.point.x
    }

    #[inline]
    pub fn y(&self) -> isize {
        self.point.y
    }

    #[inline]
    pub fn inc_edge(&self) -> Option<EdgeId> {
        self.inc_edge
    }

    #[inline]
    pub fn out_edge(&self) -> Option<EdgeId> {
        self.out_edge
    }

    #[inline]
    pub fn take_in_edge(&mut self) -> Option<EdgeId> {
        self.inc_edge.take()
    }

    #[inline]
    pub fn take_out_edge(&mut self) -> Option<EdgeId> {
        self.out_edge.take()
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.point.eq(&other.point)
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.point.cmp(&other.point)
    }
}

impl Eq for Node {}
