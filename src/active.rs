use std::fmt::Debug;

use crate::{edge::Edge, node::Node};

pub trait ActiveVec<'a>: Clone + Debug + Default {
    type Item;

    fn cursor(&self) -> usize;

    fn items(&self) -> &Vec<&Self::Item>;

    fn set_cursor(&mut self, new: usize);

    #[inline]
    fn increment(&mut self) {
        self.set_cursor(self.cursor() + 1);
    }

    #[inline]
    fn peek_at(&self, ix: usize) -> Option<&Self::Item> {
        self.items().get(ix).copied()
    }

    #[inline]
    fn peek(&self) -> Option<&Self::Item> {
        self.peek_at(self.cursor())
    }

    /// If the next item exists, return it, and increment the cursor.
    #[inline]
    fn next(&mut self) -> Option<&Self::Item> {
        let item = self.peek();
        if item.is_some() {
            self.increment();
        }
        item
    }

    /// Check if the next item exists, and then if it
    /// additionally passes the predicate supplied by the user.
    #[inline]
    fn next_if<F: FnOnce(&Self::Item) -> bool>(
        &mut self,
        f: F,
    ) -> Option<&Self::Item> {
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L229-232
        if let Some(item) = self.peek() {
            if f(item) {
                self.increment();
                return Some(item);
            }
        }
        None
    }

    fn insert(&mut self, item: &Self::Item);

    /// Reset the cursor back to the start.
    fn reset(&mut self) {
        //Based on: the various places where .begin is seen in the
        //
        // https://github.com/search?q=repo%3Abzm3r%2FOpenROAD+path%3Apoly_decomp.cpp+begin&type=code
        self.set_cursor(0);
    }

    fn with_capacity(capacity: usize) -> Self;

    #[inline]
    fn len(&'a self) -> usize {
        self.items().len()
    }

    #[inline]
    fn finished(&'a self) -> bool {
        self.len() == self.cursor()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ActiveNodes<'a> {
    nodes: Vec<&'a Node<'a>>,
    cursor: usize,
}

impl<'a> ActiveNodes<'a> {
    pub fn sort(&mut self) {
        self.nodes.sort()
    }

    pub fn scanline(&self) -> Option<isize> {
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L205
        self.peek().and_then(|node| node.y().into())
    }

    pub fn finished(&self) -> bool {
        self.cursor == self.nodes.len()
    }
}

impl<'a> ActiveVec<'a> for ActiveNodes<'a> {
    type Item = Node<'a>;

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn items(&self) -> &Vec<&Self::Item> {
        &self.nodes
    }

    fn set_cursor(&mut self, new: usize) {
        self.cursor = new;
    }

    fn insert(&mut self, item: &Self::Item) {
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

#[derive(Clone, Debug, Default)]
pub struct ActiveEdges<'a> {
    pub edges: Vec<&'a Edge<'a>>,
    cursor: usize,
}

impl<'a> ActiveEdges<'a> {
    fn maybe_insert(&mut self, edge: Option<&'a Edge<'a>>) {
        if let Some(edge) = edge {
            self.insert(edge);
        }
    }

    pub fn insert_edges_of(&mut self, node: &Node) {
        self.maybe_insert(node.in_edge());
        self.maybe_insert(node.out_edge());
    }

    /// Retain those elements which pass `f`, otherwise delete the rest.
    pub fn retain_if<F: FnMut(&&'a Edge<'a>) -> bool>(&mut self, f: F) {
        self.edges.retain(f);
    }
}

impl<'a> ActiveVec<'a> for ActiveEdges<'a> {
    type Item = Edge<'a>;

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn items(&self) -> &Vec<&Self::Item> {
        &self.edges
    }

    fn set_cursor(&mut self, new: usize) {
        self.cursor = new;
    }

    // Based on: https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L242
    fn insert(&mut self, item: &Self::Item) {
        let x = item.src_x();

        while let Some(edge) = self.next() {
            if x < edge.src_x() {
                self.edges.insert(self.cursor(), item);
                return;
            }
        }

        self.edges.push(item);
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            edges: Vec::with_capacity(capacity),
            cursor: 0,
        }
    }
}

impl<'a> FromIterator<&'a Node<'a>> for ActiveNodes<'a> {
    fn from_iter<Iterable: IntoIterator<Item = &'a Node<'a>>>(
        nodes: Iterable,
    ) -> Self {
        Self {
            nodes: nodes.into_iter().collect(),
            cursor: 0,
        }
    }
}

impl<'a> FromIterator<&'a Edge<'a>> for ActiveEdges<'a> {
    fn from_iter<Iterable: IntoIterator<Item = &'a Edge<'a>>>(
        edges: Iterable,
    ) -> Self {
        Self {
            edges: edges.into_iter().collect(),
            cursor: 0,
        }
    }
}
