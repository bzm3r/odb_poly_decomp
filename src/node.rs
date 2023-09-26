use std::{cell::Cell, cmp::Ordering};

use crate::{edge::Edge, point::Point, scanner::Side};

#[derive(Clone, Debug, Default)]
pub struct Node<'a> {
    pub point: Point,
    inc_edge: Cell<Option<&'a Edge<'a>>>,
    out_edge: Cell<Option<&'a Edge<'a>>>,
}

impl<'a> Node<'a> {
    pub fn new(
        point: Point,
        inc_edge: Option<&'a Edge<'a>>,
        out_edge: Option<&'a Edge<'a>>,
    ) -> Self {
        Self {
            point,
            inc_edge: inc_edge.into(),
            out_edge: out_edge.into(),
        }
    }

    #[inline]
    pub fn set_inc_edge(&'a self, inc: &Edge) {
        self.inc_edge.replace(Some(inc));
    }

    #[inline]
    pub fn set_out_edge(&'a self, out: &Edge) {
        self.out_edge.replace(Some(out));
    }

    #[inline]
    pub fn which_side(&self, other: &'a Node) -> Option<Side> {
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

impl<'a> From<Point> for Node<'a> {
    fn from(point: Point) -> Self {
        Node {
            point,
            inc_edge: None.into(),
            out_edge: None.into(),
        }
    }
}

impl<'a> PartialOrd for Node<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.point.partial_cmp(&other.point)
    }
}

impl<'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.point.eq(&other.point)
    }
}

impl<'a> Ord for Node<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.point.cmp(&other.point)
    }
}

impl<'a> Eq for Node<'a> {}
