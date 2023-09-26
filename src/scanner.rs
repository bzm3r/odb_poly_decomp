use std::{error::Error, fmt::Display};

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
/// Each edge is a `Segment`: which is a source point, along with some optional,
/// edge defining information.
#[derive(Clone, Debug, Default)]
pub struct Scanner<'a> {
    nodes: Vec<Node<'a>>,
    edges: Vec<Edge<'a>>,
    active_nodes: ActiveNodes<'a>,
    active_edges: ActiveEdges<'a>,
    scanline: isize,
}

impl<'a> Scanner<'a> {
    #[inline]
    fn empty(n_points: usize) -> Self {
        let capacity = 2 * n_points;
        Scanner {
            nodes: Vec::with_capacity(capacity),
            edges: Vec::with_capacity(capacity),
            active_nodes: ActiveNodes::with_capacity(capacity),
            active_edges: ActiveEdges::with_capacity(capacity),
            scanline: 0,
        }
    }

    #[inline]
    fn node(&self, ix: usize) -> &Node {
        &self.nodes[ix]
    }

    #[inline]
    fn edge(&self, ix: usize) -> &Edge {
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
        in_edge: Option<&Edge>,
        out_edge: Option<&Edge>,
        active: bool,
    ) -> &Node {
        self.nodes.push(Node::new(point, in_edge, out_edge));
        let node = self.nodes.last().unwrap();
        if active {
            self.active_nodes.insert(node);
        }
        node
    }

    fn new_edge(&mut self, source: &Node, target: &Node, side: Side) -> &Edge {
        let edge = Edge::new(source, target, side);
        self.edges.push(edge);
        let edge = self.edges.last().unwrap();
        source.set_out_edge(edge);
        target.set_inc_edge(edge);
        edge
    }

    // Based on: it is called add_edges in the original, but this is a misnomer
    // as it is specifically adding to the active edges. We only add to edges
    // during the scan_edges phase.
    // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L222
    fn add_active_edges(&mut self) {
        // Based on: 1) iterate on active_nodes, based on wherever it is
        // currently and 2) if the current node's y-marker != scanline, then
        // we should stop, as we have finished with the set of edges relevant
        // for this scanline
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L226-232
        while let Some(node) =
            self.active_nodes.next_if(|node| node.y() == self.scanline)
        {
            // Based on: add this node's edges to the active edge list
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L234-238
            self.active_edges.insert_edges_of(node);
        }
    }

    fn scan_side_edge(&mut self, required: Side) -> Option<(&Edge, usize)> {
        // Based on the general shape of:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L268-277

        // If active_edges.next() returns None (i.e. active_edges' cursor has
        // reached the end), then this function will return `None`, which will
        // cause scan edges to return as well through use of the `?` operator.
        while let Some(edge) = self.active_edges.next() {
            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L273-276
            if edge.side == required && edge.src_y() != self.scanline {
                return Some((edge, self.active_edges.cursor()));
            }
        }
        None
    }

    fn scan_edges(&mut self, mut rects: Vec<Rect>) -> Vec<Rect> {
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L258-320

        // See comment in `purge_active_edges`
        self.active_edges.reset();

        let mut left_cursor;
        let mut right_cursor;
        let mut left;
        let mut right;

        while self.active_edges.finished() {
            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L265-277
            (left, left_cursor) = if let Some((left, left_cursor)) =
                self.scan_side_edge(Side::Left)
            {
                (left, left_cursor)
            } else {
                // TODO: should we be panicking here?
                return rects;
            };

            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L279-290
            (right, right_cursor) = if let Some((right, right_cursor)) =
                self.scan_side_edge(Side::Right)
            {
                (right, right_cursor)
            } else {
                // TODO: should we be panicking here?
                return rects;
            };

            if left.inside_y(self.scanline) && right.inside_y(self.scanline) {
                // https://stackoverflow.com/a/1813008/3486684 sigh, C++
                left_cursor += 1;
                if left_cursor == right_cursor {
                    continue;
                }
            }

            if left.inside_y(self.scanline) {
                // The existing edge is not deleted, so its sufficient to do
                // an overwrite of the variable that used to contain it?
                left = self.split_edge(left, Side::Left);
            }

            if right.inside_y(self.scanline) {
                right = self.split_edge(right, Side::Right);
            }

            rects.push(Rect::new(
                left.source().unwrap().point,
                right.source().unwrap().point,
            ));
        }
        rects
    }

    // This is way too symmetric to not be simplified. Idea should be:
    // edge methods should take `side` in order to return source/target or
    // set source/target appropriately. (Essentially, must view an edges
    // end points not only as source/target, but *just* as endpoints, which
    // are then split).
    fn split_edge(&mut self, edge: &Edge, side: Side) -> &Edge {
        // "split intersected edge"
        let existing_node = match side {
            Side::Left => edge.source(),
            Side::Right => edge.target(),
        }
        .unwrap();

        // Based on: https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L300
        // confirm that `active` should be false for the following call:
        let new_node = self.new_node(
            Point::new(existing_node.x(), self.scanline),
            None,
            None,
            false,
        );

        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L301-303
        match side {
            Side::Left => {
                edge.set_source(new_node);
                existing_node.take_out_edge();
                new_node.set_out_edge(edge);
                self.new_edge(existing_node, new_node, side)
            }
            Side::Right => {
                edge.set_target(new_node);
                existing_node.take_in_edge();
                new_node.set_inc_edge(edge);
                self.new_edge(new_node, existing_node, side)
            }
        }
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
    fn update_scanline(&mut self) {
        self.scanline = self.active_nodes.scanline().unwrap();
    }

    /// Purge active edge vector.
    #[inline]
    fn purge_active_edges(&mut self) {
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L322-333
        self.active_edges
            .retain_if(|edge| edge.contains_y(self.scanline));
        // Now that we have purged this edge iterator, its cursor is invalid.
        // The original code is set up so that `add_edges` is called next
        // if the scan/decompose loop still runs. The first thing `add_edges`
        // does is set the cursor of active edges to 0...
        // TODO: Maybe we should be putting cursor in an `Option`, and then
        // setting it to none as a consequence of this operation?
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
            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#203
            scanner.active_nodes.sort();
            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#205
            scanner.update_scanline();

            Ok(scanner)
        }
    }

    pub fn decompose(&mut self) -> Vec<Rect> {
        // TODO: figure out whether its worth pre-allocating rects. If yes, then
        // what value should we pick? Currently just picked 2 * n_points...
        //
        // Note however that n_points is just the initial number of points
        // forming the rectilinear polygon. New nodes might be added. How many
        // more? Suppose each edge sees one split, then we will end up having
        // 2 * n_points, but is this an absolute upper bound? Can there be an
        // edge which is split more than once? Yes, so this is not an upper
        // bound, but just an initial guess.
        //
        // How many rects then? An absolute upper bound for the number of
        // rectangles that can exist in the decomposition is 4 * n_nodes. This
        // is because each node can only be the corner of 4 different rects.
        //
        // However in our actual scenario, all our nodes are *border* nodes.
        // This suggests that each node can have at most 3 rects. So, we get: 3
        // * n_nodes = (6 * n_points) rects
        //
        // Worth pre-allocating? Not sure.
        let mut rects = Vec::with_capacity(self.nodes.capacity());
        loop {
            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L208
            self.add_active_edges();

            rects = self.scan_edges(rects);

            if self.active_nodes.finished() {
                break;
            } else {
                // TODO: do we need something that does what line 214 does:
                // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L214

                // Based on:
                // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L215
                self.update_scanline();
                // Based on:
                // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L216
                self.purge_active_edges();
            }
        }
        rects
    }
}
