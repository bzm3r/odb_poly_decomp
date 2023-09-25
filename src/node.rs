use std::{cell::Cell, cmp::Ordering};

use crate::{edge::Edge, point::Point, scanner::Side};

#[derive(Clone, Debug, Default)]
pub struct Node<'a> {
    point: Point,
    in_edge: Cell<Option<&'a Edge<'a>>>,
    out_edge: Cell<Option<&'a Edge<'a>>>,
}

impl<'a> Node<'a> {
    #[inline]
    pub fn set_in_edge(&self, inc: &'a Edge) {
        self.in_edge.replace(Some(inc));
    }

    #[inline]
    pub fn set_out_edge(&self, out: &'a Edge) {
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
    pub fn in_edge(&self) -> Option<&Edge<'_>> {
        self.in_edge.get()
    }

    #[inline]
    pub fn out_edge(&self) -> Option<&Edge<'_>> {
        self.out_edge.get()
    }

    // Based on: 
    #[inline]
    pub fn split_node(point: Point, scanline: isize) -> Self {
        Self {
            point: Point::new(point.x, scanline),
            ..Node::default()
        }
    }

    #[inline]
    pub fn transfer_out_edge_to(&self, other: &Node<'a>) {
        let this = self.out_edge.take();
        other.out_edge.replace(this);
    }
}

impl<'a> From<Point> for Node<'a> {
    fn from(point: Point) -> Self {
        Node {
            point,
            in_edge: None.into(),
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
