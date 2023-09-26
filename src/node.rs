use std::{cmp::Ordering, ops::Deref};

use crate::{
    edge::{Edge, Edges},
    index::{EdgeIdx, NodeIdx},
    point::Point,
    scanner::Side,
};

#[derive(Clone, Debug, Default)]
pub struct Nodes<'a>(Vec<Node<'a>>);

impl<'a> Nodes<'a> {
    pub fn get(&self, idx: &NodeIdx) -> Option<&'a Node<'a>> {
        let ix: Option<usize> = idx.into();
        ix.and_then(|ix| self.0.get(ix))
    }
}

#[derive(Clone, Debug, Default)]
pub struct Node<'a> {
    edge_vec: &'a Edges<'a>,
    pub point: Point,
    inc_edge: EdgeIdx,
    out_edge: EdgeIdx,
}

impl<'a> Node<'a> {
    pub fn new(
        point: Point,
        inc_edge: Option<EdgeIdx>,
        out_edge: Option<EdgeIdx>,
    ) -> Self {
        Self {
            point,
            inc_edge: inc_edge.into(),
            out_edge: out_edge.into(),
        }
    }

    #[inline]
    pub fn set_inc_edge(&self, inc: &Edge) {
        self.inc_edge.replace(Some(inc));
    }

    #[inline]
    pub fn set_out_edge(&self, out: &Edge) {
        self.out_edge.replace(Some(out));
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
    pub fn in_edge(&self) -> Option<&Edge> {
        self.inc_edge.get()
    }

    #[inline]
    pub fn out_edge(&self) -> Option<&Edge> {
        self.out_edge.get()
    }

    #[inline]
    pub fn take_in_edge(&self) -> Option<&Edge> {
        self.inc_edge.take()
    }

    #[inline]
    pub fn take_out_edge(&self) -> Option<&Edge> {
        self.out_edge.take()
    }
}

impl From<Point> for Node {
    fn from(point: Point) -> Self {
        Node {
            point,
            inc_edge: None.into(),
            out_edge: None.into(),
        }
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.point.partial_cmp(&other.point)
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
