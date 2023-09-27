use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

use id_arena::Id;

use crate::{
    edge::{Edge, EdgeId},
    geometry::{GeometricId, Geometry},
    node::{Node, NodeId},
};

#[allow(clippy::len_without_is_empty)]
pub trait ActiveVec
where
    Self: Clone + Debug + Default,
    Geometry: Index<Self::Id, Output = Self::Item> + IndexMut<Self::Id>,
{
    type Item: Clone + Copy;
    type Id: GeometricId<Item = Self::Item> + Clone + Copy;

    fn cursor(&self) -> usize;

    fn items(&self) -> &Vec<Self::Id>;

    fn set_cursor(&mut self, new: Cursor);

    #[inline]
    fn increment(&mut self) {
        self.set_cursor(self.cursor() + 1);
    }

    #[inline]
    fn peek_at(&self, ix: usize) -> Option<Self::Id> {
        self.items().get(ix).copied()
    }

    #[inline]
    fn peek(&self) -> Option<Self::Id> {
        self.peek_at(self.cursor())
    }

    /// If the next item exists, return it, and increment the cursor.
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
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L229-232
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
}

#[derive(Clone, Default)]
pub struct ActiveNodes {
    nodes: Vec<Id<Node>>,
    cursor: Cursor,
}

impl Debug for ActiveNodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.nodes
                .iter()
                .map(|id| id.index().to_string())
                .fold(String::new(), |a, b| format!("{}, {}", a, b))
        )
    }
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
}

#[derive(Clone, Default)]
pub struct ActiveEdges {
    pub edges: Vec<EdgeId>,
    cursor: Cursor,
}

impl Debug for ActiveEdges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.edges
                .iter()
                .map(|id| id.index().to_string())
                .fold(String::new(), |a, b| format!("{}, {}", a, b))
        )
    }
}

impl ActiveEdges {
    fn maybe_insert(&mut self, geometry: &Geometry, edge_id: Option<EdgeId>) {
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
    type Id = EdgeId;
    type Item = Edge;

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn items(&self) -> &Vec<Self::Id> {
        &self.edges
    }

    fn set_cursor(&mut self, new: Cursor) {
        self.cursor = new;
    }

    // Based on: https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L242
    fn insert(&mut self, geometry: &Geometry, id: Self::Id) {
        let x = geometry[id].src_x(geometry);

        while let Some(_edge) = self.next(geometry) {
            if x < geometry[id].src_x(geometry) {
                self.edges.insert(self.cursor(), id);
                return;
            }
        }

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
