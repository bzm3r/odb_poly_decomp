use crate::{decomposer::DecompErr, edge::Edge, node::Node, point::Point};

pub struct Geometry {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Geometry {
    fn empty(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            edges: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn new_node(
        &mut self,
        point: Point,
        in_edge: Option<&Edge>,
        out_edge: Option<&Edge>,
    ) -> &Node {
        self.nodes.push(Node::new(point, in_edge, out_edge));
        let node = self.nodes.last().unwrap();
        node
    }

    pub fn new_edge(
        &mut self,
        source: &Node,
        target: &Node,
        side: Side,
    ) -> &Edge {
        let edge = Edge::new(source, target, side);
        self.edges.push(edge);
        let edge = self.edges.last().unwrap();
        source.set_out_edge(edge);
        target.set_inc_edge(edge);
        edge
    }

    #[inline]
    pub fn get_node(&self, ix: usize) -> &Node {
        &self.nodes[ix]
    }

    #[inline]
    pub fn get_edge(&self, ix: usize) -> &Edge {
        &self.edges[ix]
    }

    #[inline]
    pub fn len_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// For use when Geometry is being initialized.
    fn initialize_nodes(&mut self, points: Vec<Point>) {
        // Based on:
        // 1) https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L179
        // 2) https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L186
        // note carefully: `true` is passed for active in call to new node
        points.into_iter().for_each(|p| {
            self.new_node(p, None, None, true);
        });
    }

    pub fn iter_nodes(&self) -> std::slice::Iter<'_, Node<'_>> {
        self.nodes.iter()
    }

    /// For use when Geometry is being intialized.
    fn initialize_edges(&mut self) {
        // Based on:
        // 1) https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L189
        // 2) https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L192
        // 3) https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L198
        // 4) https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L201
        let n_nodes = self.len_nodes();
        debug_assert!(self.len_nodes() > 3);
        let (s, t) = (0, 1);
        loop {
            let (source, target) = (&self.get_node(s), &self.get_node(t));
            if let Some(side) = source.which_side(target) {
                self.new_edge(&source, &target, side);
            }
            let (s, t) = (t, (t + 1) % n_nodes);
        }
    }

    pub fn iter_edges(&self) -> std::slice::Iter<'_, Node<'_>> {
        self.nodes.iter()
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
        if points.len() > 3 {
            let mut geometry = Self::empty(points.len());
            geometry.initialize_nodes(points);
            geometry.initialize_edges();
            Ok(geometry)
        } else {
            Err(DecompErr::NotEnoughPoints)
        }
    }
}
