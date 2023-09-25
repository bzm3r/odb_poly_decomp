use edge::Edge;
use point::Point;
use rect::Rect;
use scanner::Scanner;

mod active;
mod edge;
mod node;
mod point;
mod rect;
mod scanner;

pub struct Decomposer<'a> {
    scanner: Scanner<'a>,
    edges: Vec<Edge<'a>>,
    node_cursor: usize,
    edge_cursor: usize,
}

impl<'a> Decomposer<'a> {
    fn add_edges(&mut self, edges: &Decomposer) {
        while let Some(edge) =  {}
    }

    fn active_edges(&self, scanline: isize) -> active_edges {
        todo!()
    }
}

impl PolyDecomp {
    fn add_edges(node_cursor: usize, scanline: isize) {}

    fn insert_edge(_edge: Edge, _edges: &mut Vec<Node>) {
        unimplemented!()
    }

    fn scan_edges(_scanline: isize, _rects: &Vec<Rect>) {
        unimplemented!()
    }

    fn purge_edges(_scanline: isize) {
        unimplemented!()
    }

    fn decompose(&mut self, points: Vec<Point>) -> Vec<Rect> {
        let mut rects = Vec::new();

        if points.len() < 4 {
            unimplemented!("need to push remaining points to rect?");
            return rects;
        } else {
            // This initializes edges with "vertical edges for scanline
            // intersection" Based on (lines: 183 to ):   // Create
            // vertical edges for scanline intersection
            let edges = Decomposer::new(points);

            unimplemented!()
        }
    }
}
