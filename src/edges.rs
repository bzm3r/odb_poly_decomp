use std::ops::Index;

use crate::point::Point;

#[derive(Clone, Copy, Debug)]
pub enum Side {
    Left,
    Right,
}

/// We only need the target point to define an edge, as the source point is
/// the point with the corresponding index in [`Edges::points`].
#[derive(Clone, Copy, Debug)]
pub struct Edge {
    target: usize,
    side: Side,
}

/// Edge structure of the rectilinear polygon.
///
/// Each edge is a `Segment`: which is a source point,
/// along with some optional, edge defining information.
#[derive(Clone, Debug)]
pub struct Edges {
    points: Vec<Point>,
    edges: Vec<Option<Edge>>,
    scan_order: Vec<usize>,
}

impl Edges {
    #[inline]
    fn get_point(&self, ix: usize) -> &Point {
        &self.points[self.scan_order[ix]]
    }

    #[inline]
    fn get_edge(&self, ix: usize) -> Option<&Edge> {
        self.edges[self.scan_order[ix]].as_ref()
    }

    /// Initialized with the vertical edges needed for scanline intersection
    /// test.
    ///
    /// It requires that the supplied polygon points are sorted in clockwise
    /// order.
    ///
    /// Based on:
    /// https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L183
    pub fn new(points: Vec<Point>) -> Self {
        let mut edges = vec![None; points.len()];
        let mut order = (0..points.len()).collect::<Vec<usize>>();

        for (source, (s, edge)) in points.iter().zip(edges.iter_mut()).enumerate() {
            let target = (source + 1) % edges.len();
            let t = &points[target];
            if let Some(side) = s.which_side(t) {
                edge.replace(Edge { target, side });
            }
        }

        order.sort_by(|&i, &j| points[i].cmp(&points[j]));

        Self {
            points,
            edges,
            scan_order: order,
        }
    }

    pub fn scan_iter(&self) -> impl Iterator<Item = &Edge> {
        unimplemented!()
    }
}

pub struct ScanIter<'a> {
    edges: &'a Edges,
    cursor: usize,
}

impl<'a> Iterator for ScanIter<'a> {
    type Item = &'a Edge;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
