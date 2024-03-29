use std::{
    fmt::Debug,
    hash::Hash,
    ops::{Index, IndexMut},
};

use id_arena::Id;

use tracing::info;

use crate::{
    edge::{Edge, EdgeId},
    geometry::{GeometricId, Geometry},
    node::{Node, NodeId},
};

#[allow(clippy::len_without_is_empty)]
pub trait ActiveVec
where
    Self: Clone + Default,
    Geometry: Index<Self::Id, Output = Self::Item> + IndexMut<Self::Id>,
{
    type Item: Clone + Copy + Debug;
    type Id: GeometricId<Item = Self::Item>
        + Clone
        + Copy
        + Hash
        + PartialEq
        + Eq;

    fn cursor(&self) -> usize;

    fn items(&self) -> &Vec<Self::Id>;

    fn set_cursor(&mut self, new: Cursor);

    #[inline]
    fn increment(&mut self) {
        self.set_cursor(self.cursor() + 1);
    }

    /// Get the item stored at the specified position, without affecting the
    /// cursor's value.
    #[inline]
    fn peek_at(&self, ix: usize) -> Option<Self::Id> {
        self.items().get(ix).copied()
    }

    /// Get the item stored at the current cursor's position, without affecting
    /// the cursor's value.
    #[inline]
    fn peek(&self) -> Option<Self::Id> {
        self.peek_at(self.cursor())
    }

    /// If the item at the current cursor's position exists, get it, and
    /// increment the cursor.
    #[inline]
    fn next(&mut self, geometry: &Geometry) -> Option<Self::Item> {
        if let Some(id) = self.peek() {
            self.increment();
            Some(geometry[id])
        } else {
            None
        }
    }

    /// Check if the next item exists, and then if it
    /// additionally passes the predicate supplied by the user.
    #[inline]
    fn next_if<F: FnOnce(&Geometry, Self::Id) -> Option<Self::Item>>(
        &mut self,
        geometry: &Geometry,
        f: F,
    ) -> Option<Self::Item> {
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L229-L232
        let result = self.peek().and_then(|id| f(geometry, id));
        if result.is_some() {
            self.increment();
        }
        result
    }

    fn insert(&mut self, geometry: &Geometry, item: Self::Id);

    /// Reset the cursor back to the start.
    fn reset_cursor(&mut self) {
        //Based on: the various places where .begin is seen in the
        //
        // https://github.com/search?q=repo%3Abzm3r%2FOpenROAD+path%3Apoly_decomp.cpp+begin&type=code
        self.set_cursor(0);
    }

    fn with_capacity(capacity: usize) -> Self;

    #[inline]
    fn len(&self) -> usize {
        self.items().len()
    }

    #[inline]
    fn finished(&self) -> bool {
        self.len() == self.cursor()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.items().is_empty()
    }
}

#[derive(Clone, Default)]
pub struct ActiveNodes {
    pub nodes: Vec<Id<Node>>,
    pub cursor: Cursor,
}

impl ActiveNodes {
    pub fn sort(&mut self, geometry: &Geometry) {
        self.nodes.sort_by(|&a, &b| geometry[a].cmp(&geometry[b]));
    }

    pub fn scanline(&self, geometry: &Geometry) -> Option<isize> {
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L205
        self.peek().and_then(|id| geometry[id].y().into())
    }

    pub fn finished(&self) -> bool {
        self.cursor == self.nodes.len()
    }
}

impl ActiveVec for ActiveNodes {
    type Id = NodeId;
    type Item = Node;

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn items(&self) -> &Vec<Self::Id> {
        &self.nodes
    }

    fn set_cursor(&mut self, new: Cursor) {
        self.cursor = new;
    }

    fn insert(&mut self, _: &Geometry, item: Self::Id) {
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/master/src/odb/src/zutil/poly_decomp.cpp#L186
        self.nodes.push(item);
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            cursor: 0,
        }
    }

    fn len(&self) -> usize {
        self.items().len()
    }

    // fn debug<'a>(&self, ) -> String {
    //     self.active_nodes
    //         .items()
    //         .iter()
    //         .enumerate()
    //         .map(|(ix, &id)| {
    //             format!(
    //                 "{:?}",
    //                 DebugItem {
    //                     style: (ix == self.active_nodes.cursor())
    //                         .then_some(STYLE_CURSOR)
    //                         .unwrap_or(MiniStyle::default()),
    //                     data: self.geometry[id],
    //                 }
    //             )
    //         })
    //         .join(", ")
    // }
}

#[derive(Clone, Default)]
pub struct ActiveEdges {
    pub edges: Vec<EdgeId>,
    pub cursor: Cursor,
}

impl ActiveEdges {
    pub fn maybe_insert(
        &mut self,
        geometry: &Geometry,
        edge_id: Option<EdgeId>,
    ) {
        if let Some(id) = edge_id {
            self.insert(geometry, id);
        }
    }

    /// Insert the incoming and outgoing edges of a node, if they exist, into
    /// the active edge vec.
    pub fn insert_edges(
        &mut self,
        geometry: &Geometry,
        inc: Option<EdgeId>,
        out: Option<EdgeId>,
    ) {
        // A node might not yet have incoming/outgoing edges set.
        self.maybe_insert(geometry, inc);
        self.maybe_insert(geometry, out);
    }

    /// Retain those elements which pass `f`, otherwise delete the rest.
    pub fn retain_if<F: FnMut(&EdgeId) -> bool>(&mut self, f: F) {
        self.edges.retain(f);
    }
}

pub type Cursor = usize;

impl ActiveVec for ActiveEdges {
    type Item = Edge;
    type Id = EdgeId;

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn items(&self) -> &Vec<Self::Id> {
        &self.edges
    }

    fn set_cursor(&mut self, new: Cursor) {
        self.cursor = new;
    }

    fn insert(&mut self, geometry: &Geometry, id: Self::Id) {
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L242-L256

        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L244
        let x = geometry[id].src_x(geometry);

        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L246-L253
        while let Some(other) = self.next(geometry) {
            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L249
            if x < other.src_x(geometry) {
                // We have to insert at where the cursor was last, because next
                // returns the item at the current position, and then
                // increments, while we want the cursor position where the
                // returned item was at.
                self.edges.insert(self.cursor() - 1, id);
                info!("adding edge to active list: {}", id.index());
                return;
            }
        }

        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L255
        // If we are here, active_edges should have reached the end. So,
        // inserting there should be equivalent to pushing onto the end of the
        // cursor.
        self.edges.push(id);
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            edges: Vec::with_capacity(capacity),
            cursor: 0,
        }
    }
}

impl FromIterator<NodeId> for ActiveNodes {
    fn from_iter<Iterable: IntoIterator<Item = NodeId>>(
        nodes: Iterable,
    ) -> Self {
        Self {
            nodes: nodes.into_iter().collect(),
            cursor: 0,
        }
    }
}

impl FromIterator<EdgeId> for ActiveEdges {
    fn from_iter<Iterable: IntoIterator<Item = EdgeId>>(
        edges: Iterable,
    ) -> Self {
        Self {
            edges: edges.into_iter().collect(),
            cursor: 0,
        }
    }
}
