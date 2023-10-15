use std::ops::{Index, IndexMut};

use id_arena::{Arena, DefaultArenaBehavior};

use tracing::info;

use crate::{
    dbg_edge, dbg_edges,
    decomposer::DecompErr,
    edge::{Edge, EdgeId},
    node::{Node, NodeId},
    point::Point,
};

pub struct Geometry {
    pub nodes: Arena<Node>,
    pub edges: Arena<Edge>,
}

impl Index<EdgeId> for Geometry {
    type Output = Edge;
    fn index(&self, id: EdgeId) -> &Self::Output {
        self.edges.get(id).unwrap()
    }
}

impl Index<NodeId> for Geometry {
    type Output = Node;
    fn index(&self, id: NodeId) -> &Self::Output {
        self.nodes.get(id).unwrap()
    }
}

impl IndexMut<EdgeId> for Geometry {
    fn index_mut(&mut self, id: EdgeId) -> &mut Self::Output {
        self.edges.get_mut(id).unwrap()
    }
}

impl IndexMut<NodeId> for Geometry {
    fn index_mut(&mut self, id: NodeId) -> &mut Self::Output {
        self.nodes.get_mut(id).unwrap()
    }
}

pub trait GeometricId
where
    Geometry: Index<Self, Output = Self::Item> + IndexMut<Self>,
{
    type Item;
    fn index(&self) -> usize;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EdgeTy {
    Left,
    Right,
}

impl Geometry {
    #[inline]
    fn empty(capacity: usize) -> Self {
        Self {
            nodes: Arena::with_capacity(capacity),
            edges: Arena::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn new_node(
        &mut self,
        point: Point,
        in_edge: Option<EdgeId>,
        out_edge: Option<EdgeId>,
    ) -> NodeId {
        self.nodes
            .alloc_with_id(|id| Node::new(point, id, in_edge, out_edge))
    }

    #[inline]
    pub fn new_edge(
        &mut self,
        source: NodeId,
        target: NodeId,
        ty: EdgeTy,
    ) -> Edge {
        let new_edge_id = self
            .edges
            .alloc_with_id(|id| Edge::new(id, source, target, ty));
        self[source].set_out_edge(new_edge_id);
        self[target].set_inc_edge(new_edge_id);
        self[new_edge_id]
    }

    #[inline]
    pub fn len_nodes(&self) -> usize {
        self.nodes.len()
    }

    fn initialize_nodes(&mut self, points: &[Point]) -> Vec<NodeId> {
        points
            .iter()
            .map(|&p| self.new_node(p, None, None))
            .collect::<Vec<NodeId>>()
    }

    /// For use when Geometry is being intialized.
    fn initialize_nodes_and_edges(&mut self, points: Vec<Point>) {
        let n_nodes = points.len();
        // Based on:
        // 1) https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L179
        let node_ids = self.initialize_nodes(&points);
        info!(
            "node_ids: {:?}",
            node_ids.iter().map(|id| id.index()).collect::<Vec<usize>>()
        );
        info!(
            "nodes: {:#?}",
            self.nodes
                .iter()
                .map(|(_id, node)| *node)
                .collect::<Vec<Node>>()
        );

        // Based on:
        // 1) https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L189
        // 2) https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L192
        // 3) https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L198
        // 4) https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L201
        let (mut s, mut t) = (0_usize, 1_usize);
        loop {
            let (source, target) = (node_ids[s], node_ids[t]);
            let source_node = self[source];
            if source_node.out_edge().is_some() {
                break;
            }
            if let Some(side) = source_node.which_side(&self[target]) {
                self.new_edge(source, target, side);
                info!(
                    "new edge: {:?}",
                    dbg_edge!(self, self.edges.iter().last().unwrap().1)
                );
            }
            (s, t) = (t, (t + 1) % n_nodes);
        }
        info!("edges: {:?}", dbg_edges!(self, &self.edges));
    }

    pub fn iter_edges(
        &self,
    ) -> id_arena::Iter<'_, Edge, DefaultArenaBehavior<Edge>> {
        self.edges.iter()
    }

    pub fn iter_nodes(
        &self,
    ) -> id_arena::Iter<'_, Node, DefaultArenaBehavior<Node>> {
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
        // The original code checks to see that there are greater than 3 nodes,
        // because it is only in that case that it is possible to have a
        // rectilinear polygon (otherwise, you have a rectangle, or even worse,
        // a triangle)
        match points.len() {
            n if n > 3 => {
                let mut geometry = Self::empty(points.len());
                geometry.initialize_nodes_and_edges(points);
                Ok(geometry)
            }
            3 => Err(DecompErr::IsAlreadySimple),
            _ => Err(DecompErr::NotEnoughPoints),
        }
    }

    // This is way too symmetric to not be simplified. Idea should be:
    // edge methods should take `side` in order to return source/target or
    // set source/target appropriately. (Essentially, must view an edges
    // end points not only as source/target, but *just* as endpoints, which
    // are then split).
    pub fn split_edge(
        &mut self,
        split_target: EdgeId,
        scanline: isize,
    ) -> Edge {
        // "split intersected edge"
        // Based on (for left edge):
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L299-304
        // Based on (for right  edge): https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#309-314

        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L299
        // and  https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L309
        // existing corresponds to u (left)/w (right) in the original code
        let side = self[split_target].ty;
        let input_node = match side {
            EdgeTy::Left => self[split_target].source,
            EdgeTy::Right => self[split_target].target,
        };
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L300
        //  https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L310
        let existing_x = self[input_node].x();

        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L300
        //  https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L310
        // TODO: confirm that the edge should not be added to active nodes list
        let new_node_id =
            self.new_node(Point::new(existing_x, scanline), None, None);

        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L301-L303
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L311-L314
        match side {
            EdgeTy::Left => {
                // input edge gets its source replaced by new node
                self[split_target].set_source(new_node_id);
                // input node (source of input edge) has its outgoing edge (the
                // input edge) deleted
                self[input_node].take_out_edge();
                // new node has its outgoing edge set to the input_edge
                self[new_node_id].set_out_edge(split_target);
                // insert a new edge into the geometry edge list
                // TODO: (Note: we do not add it to the active edge list, since
                // we do not want to split it? No: it's because we will call
                // add_edges again later, that should manage adding in new edges
                // into the active edge list, if it still needs to be split.)
                // Based on:
                // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L304
                self.new_edge(input_node, new_node_id, side)
            }
            EdgeTy::Right => {
                // input edge gets its target replaced by new node
                self[split_target].set_target(new_node_id);
                // input node (target of input edge) has its incoming edge
                // (the input edge) deleted
                self[input_node].take_inc_edge();
                // new node has its incoming edge set to be the input edge
                self[new_node_id].set_inc_edge(split_target);
                // TODO: (Note: we do not add it to the active edge list, since
                // we do not want to split it? No: it's because we will call
                // add_edges again later, that should manage adding in new edges
                // into the active edge list, if it still needs to be split.)
                // Based on:
                // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L314
                self.new_edge(new_node_id, input_node, side)
            }
        }
    }
}
