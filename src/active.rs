use std::fmt::Debug;

use crate::{edge::Edge, node::Node};

pub trait ActiveVec<'a>: Clone + Debug + Default {
    type Item;

    fn cursor(&self) -> usize;

    fn items(&self) -> &Vec<&'a Self::Item>;

    fn set_cursor(&mut self, new: usize);

    #[inline]
    fn increment(&mut self) {
        self.set_cursor(self.cursor() + 1);
    }

    #[inline]
    fn get(&self, ix: usize) -> Option<&'a Self::Item> {
        self.items().get(ix).copied()
    }

    #[inline]
    fn get_current(&self) -> Option<&'a Self::Item> {
        self.get(self.cursor())
    }

    #[inline]
    fn next(&mut self) -> Option<&'a Self::Item> {
        let item = self.get_current();
        if item.is_some() {
            self.increment();
        }
        item
    }

    fn peek(&self) -> Option<&'a Self::Item> {
        self.get(self.cursor() + 1)
    }

    fn next_if<F: FnOnce(&'a Self::Item) -> bool>(
        &self,
        f: F,
    ) -> Option<&'a Self::Item> {
        self.peek().and_then(|item| f(item).then_some(item))
    }

    fn insert(&mut self, item: &'a Self::Item);

    fn reset(&self) {
        self.set_cursor(0);
    }

    fn with_capacity(capacity: usize) -> Self;
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

    // Based on: https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L205
    pub fn scanline(&self) -> Option<isize> {
        self.get_current().and_then(|node| node.y().into())
    }
}

impl<'a> ActiveVec<'a> for ActiveNodes<'a> {
    type Item = Node<'a>;

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn items(&self) -> &Vec<&'a Self::Item> {
        &self.nodes
    }

    fn set_cursor(&mut self, new: usize) {
        self.cursor = new;
    }

    fn insert(&mut self, item: &'a Self::Item) {
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
    edges: Vec<&'a Edge<'a>>,
    cursor: usize,
}

impl<'a> ActiveEdges<'a> {
    fn maybe_insert(&self, edge: Option<&'a Edge<'a>>) {
        if let Some(edge) = edge {
            self.insert(edge);
        }
    }

    pub fn insert_node_edges(&mut self, node: &'a Node<'a>) {
        self.maybe_insert(node.in_edge());
        self.maybe_insert(node.out_edge());
    }
}

impl<'a> ActiveVec<'a> for ActiveEdges<'a> {
    type Item = Edge<'a>;

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn items(&self) -> &Vec<&'a Self::Item> {
        &self.edges
    }

    fn set_cursor(&mut self, new: usize) {
        self.cursor = new;
    }

    // Based on: https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L242
    fn insert(&mut self, item: &'a Self::Item) {
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
