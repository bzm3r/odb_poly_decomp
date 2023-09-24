use edges::Edges;
use point::Point;
use rect::Rect;

mod edges;
mod point;
mod rect;

pub struct PolyDecomp {}

impl PolyDecomp {
    fn add_edges(_nodes: &mut Vec<Node>, _scanline: isize) {
        unimplemented!()
    }

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
            // This initializes edges with "vertical edges for scanline intersection"
            // Based on (lines: 183 to ):   // Create vertical edges for scanline intersection
            let edges = Edges::new(points);

            unimplemented!()
        }
    }
}
