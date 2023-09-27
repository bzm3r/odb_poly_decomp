use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

use id_arena::{Arena, DefaultArenaBehavior};

use tracing::info;

use crate::{
    decomposer::DecompErr,
    edge::{Edge, EdgeId},
    node::{Node, NodeId},
    point::Point,
};

pub struct Geometry {
    nodes: Arena<Node>,
    edges: Arena<Edge>,
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
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Debug for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Side::Left => "LEFT",
                Side::Right => "RIGHT",
            }
        )
    }
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
        side: Side,
    ) -> Edge {
        let new_edge_id = self
            .edges
            .alloc_with_id(|id| Edge::new(id, source, target, side));
        self[source].set_out_edge(new_edge_id);
        self[target].set_inc_edge(new_edge_id);
        self[new_edge_id]
    }

    #[inline]
    pub fn len_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// For use when Geometry is being intialized.
    fn initialize_nodes_and_edges(&mut self, points: Vec<Point>) {
        let n_nodes = points.len();
        debug_assert!(n_nodes > 3);
        // Based on:
        // 1) https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L179
        let node_ids = points
            .into_iter()
            .map(|p| self.new_node(p, None, None))
            .collect::<Vec<NodeId>>();
        info!(
            "node_ids: {:?}",
            node_ids.iter().map(|id| id.index()).collect::<Vec<usize>>()
        );
        info!(
            "nodes: {:?}",
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
                info!(
                    "creating edge between: {:?} and  {:?}",
                    source.index(),
                    target.index()
                );
                self.new_edge(source, target, side);
            }
            (s, t) = (t, (t + 1) % n_nodes);
        }
        info!(
            "edges: {:?}",
            self.edges
                .iter()
                .map(|(_id, edge)| *edge)
                .collect::<Vec<Edge>>()
        );
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
        if points.len() > 3 {
            let mut geometry = Self::empty(points.len());
            geometry.initialize_nodes_and_edges(points);
            Ok(geometry)
        } else {
            Err(DecompErr::NotEnoughPoints)
        }
    }
}
