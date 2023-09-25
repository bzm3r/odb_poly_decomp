use std::{error::Error, fmt::Display, ops::Deref};

use crate::{
    active::{ActiveEdges, ActiveNodes, ActiveVec},
    edge::Edge,
    node::Node,
    point::Point,
    rect::Rect,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecompErr {
    NotEnoughPoints,
}

impl Display for DecompErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for DecompErr {}

/// Edge structure of the rectilinear polygon.
///
/// Each edge is a `Segment`: which is a source point,
/// along with some optional, edge defining information.
#[derive(Clone, Debug, Default)]
pub struct Scanner<'a> {
    nodes: Vec<Node<'a>>,
    edges: Vec<Edge<'a>>,
    active_nodes: ActiveNodes<'a>,
    active_edges: ActiveEdges<'a>,
    rects: Vec<Rect>,
    scanline: isize,
}

pub type NodeRef<'a> = &'a Node<'a>;
pub type EdgeRef<'a> = &'a Edge<'a>;

impl<'a> Scanner<'a> {
    #[inline]
    fn empty(n_points: usize) -> Self {
        let capacity = 2 * n_points;
        Scanner {
            nodes: Vec::with_capacity(capacity),
            edges: Vec::with_capacity(capacity),
            active_nodes: ActiveNodes::with_capacity(capacity),
            active_edges: ActiveEdges::with_capacity(capacity),
            rects: Vec::with_capacity(capacity),
            scanline: None,
        }
    }

    #[inline]
    fn node(&self, ix: usize) -> &Node<'a> {
        &self.nodes[ix]
    }

    #[inline]
    fn edge(&self, ix: usize) -> EdgeRef<'a> {
        &self.edges[ix]
    }

    #[inline]
    fn get<T, F: Fn(&Scanner, usize) -> T>(&self, f: F, ix: usize) -> T {
        f(&self, ix)
    }

    #[inline]
    fn get_ordered<T, F: Fn(&Scanner, usize) -> T>(
        &self,
        f: F,
        order: Vec<usize>,
        ix: usize,
    ) -> T {
        self.get(f, order[ix])
    }

    #[inline]
    fn new_node(
        &mut self,
        point: Point,
        in_edge: Option<EdgeRef<'a>>,
        out_edge: Option<EdgeRef<'a>>,
        active: bool,
    ) -> NodeRef<'a> {
        self.nodes.push(Node::new(point, in_edge, out_edge));
        let node = self.nodes.last().unwrap();
        if active {
            self.active_nodes.insert(node);
        }
        node
    }

    fn new_edge(
        &mut self,
        source: NodeRef<'a>,
        target: NodeRef<'a>,
        side: Side,
    ) {
        let edge = Edge::new(source, target, side);
        source.set_out_edge(&edge);
        target.set_in_edge(&edge);
        self.edges.push(edge);
    }

    // Based on:
    // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L222
    fn add_active_edges(&mut self) {
        // TODO: should this be an unwrap? what happens if add_active_edges
        // is called but there are no nodes? If this is not possible, why?
        // Who maintains this invariant?
        let scanline = self.active_nodes.scanline().unwrap();
        while let Some(node) =
            self.active_nodes.next_if(|node| node.y() == scanline)
        {
            self.active_edges.insert_node_edges(node);
        }
    }

    fn scan_side_edge(&self, required: Side) -> Option<(EdgeRef<'a>, usize)> {
        while let Some(edge) = self.active_edges.next() {
            if edge.side == required && edge.src_y() != self.scanline {
                return Some((edge, self.active_edges.cursor()));
            }
        }
        None
    }

    fn scan_edges(&self) -> Result<(), ()> {
        self.active_edges.reset();
        while let Some(ae) = self.active_edges.next() {
            let (left, mut left_cursor) =
                self.scan_side_edge(Side::Left).ok_or(())?;

            let (right, right_cursor) =
                self.scan_side_edge(Side::Right).ok_or(())?;

            let scanline = self.scanline;
            if left.inside_y(scanline) && right.inside_y(scanline) {
                // https://stackoverflow.com/a/1813008/3486684
                // sigh, C++
                left_cursor += 1;
                if left_cursor == right_cursor {
                    continue;
                }
            } else if left.inside_y(scanline) {
                let u = *left.source;
                let v = Node::from(Point::new(u.x(), scanline));
                u.out_edge_transfer_to(&v);
                left.source = v;
            }
        }
        unimplemented!()
    }

    /// For use when a scanner is being initialized.
    fn initialize_nodes(&mut self, points: Vec<Point>) {
        // Based on:
        // 1) https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L179
        // 2) https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L186
        // note carefully: `true` is passed for active in call to new node
        points.into_iter().for_each(|p| {
            self.new_node(p, None, None, true);
        });
    }

    /// For use when a scanner is being intialized.
    fn initialize_edges(&mut self) {
        // Based on:
        // 1) https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L189
        // 2) https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L192
        // 3) https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L198
        // 4) https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L201
        let n_nodes = self.nodes.len();
        debug_assert!(self.nodes.len() > 3);
        let (s, t) = (0, 1);
        loop {
            let (source, target) = (&self.nodes[s], &self.nodes[t]);
            if let Some(side) = source.which_side(target) {
                self.new_edge(&source, &target, side);
            }
            let (s, t) = (t, (t + 1) % n_nodes);
        }
    }

    #[inline]
    fn update_scanline(&self) {
        self.scanline = self.active_nodes.scanline().unwrap();
    }

    /// Initialized with the vertical edges needed for scanline intersection
    /// test.
    ///
    /// It requires that the supplied polygon points are sorted in clockwise
    /// order.
    ///
    /// Based on:
    /// https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L183
    pub fn new(points: Vec<Point>) -> Result<Self, DecompErr> {
        if points.len() < 4 {
            return Err(DecompErr::NotEnoughPoints);
        } else {
            let mut scanner = Self::empty(points.len());
            scanner.initialize_nodes(points);
            scanner.initialize_edges();
            // Based on: https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#203
            scanner.active_nodes.sort();
            // Based on: https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#204
            scanner.update_scanline();

            Ok(scanner)
        }
    }
}
