use crate::{
    active::{ActiveEdges, ActiveNodes, ActiveVec},
    edge::Edge,
    node::Node,
    point::Point,
};

#[derive(Clone, Copy, Debug)]
pub enum Side {
    Left,
    Right,
}

/// Edge structure of the rectilinear polygon.
///
/// Each edge is a `Segment`: which is a source point,
/// along with some optional, edge defining information.
#[derive(Clone, Debug)]
pub struct Scanner<'a> {
    nodes: Vec<Node<'a>>,
    edges: Vec<Option<Edge<'a>>>,
    active_nodes: ActiveNodes<'a>,
    active_edges: ActiveEdges<'a>,
    rects: Vec<Rect>,
}

impl<'a> Scanner<'a> {
    #[inline]
    fn node(&self, ix: usize) -> &Node<'a> {
        &self.nodes[ix]
    }

    #[inline]
    fn edge(&self, ix: usize) -> Option<&Edge<'a>> {
        self.edges[ix].as_ref()
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

    // Based on:
    // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L222
    fn add_active_edges(&mut self) {
        while let Some(node) =
            self.active_nodes.next_if(|node| node.y() == self.scanline)
        {
            self.active_edges.insert_node_edges(node);
        }
        // for node_ix in self.cursor..self.active_nodes.len() {
        //     let node = self.nodes[node_ix];
        //     if node.point.y != self.scanline {
        //         break;
        //     }
        //     self.cursor = node_ix;

        //     self.push_active_edge(node.in_edge);
        //     self.push_active_edge(node.out_edge);
        // }
    }

    fn scan_edges(&self) {

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
        let nodes = points
            .into_iter()
            .map(|point| Node::from(point))
            .collect::<Vec<Node>>();

        let mut active_nodes = ActiveNodes::with_capacity(nodes.len());
        let active_edges = ActiveEdges::with_capacity(nodes.len());

        // Note: L179 and L186 suggest that all points are initially marked as
        // active. Really, "w" only exists in order to "close the loop" of the
        // polygon. So we just iterate over all the points given to us, and use
        // some modulo math to get the next point's index regardless of what the
        // current vertex is.
        for (src, (source, edge)) in
            nodes.iter().zip(edges.iter_mut()).enumerate()
        {
            let tgt = (src + 1) % edges.len();
            let target = &nodes[tgt];
            active_nodes.insert(target);
            if let Some(side) = source.which_side(target) {
                edge.replace(Edge::new(source, target, side));
            }
        }

        active_nodes.sort();
        let scanline = active_nodes.scanline().unwrap();

        Self {
            nodes,
            edges,
            active_nodes,
            active_edges,
            edge_cursor: 0,
            cursor: 0,
            scanline,
        }
    }
}
