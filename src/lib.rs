use edge::{Edge, Side};
use node::Node;
use point::Point;
use rect::Rect;

mod edge;
mod node;
mod point;
mod rect;

// #[cfg(test)] mod tests { use super::*;

//     #[test] fn it_works() { let result = add(2, 2); assert_eq!(result, 4); }
// }

pub struct PolyDecomp<'a> {
    nodes: Vec<Node<'a>>,
    edges: Vec<Edge<'a>>,
}

impl<'a> PolyDecomp<'a> {
    // Based on:
    // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L146
    fn new_node(&mut self, point: Point) -> &'a Node {
        self.nodes.push(Node::from(point));
        self.nodes.last().unwrap()
    }

    // Based on:
    // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L156
    fn new_edge(&'a mut self, src: &'a Node<'a>, tgt: &'a Node<'a>, side: Side) {
        self.edges.push(Edge::new(src, tgt, side))
    }

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

    fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
    }

    fn decompose(&mut self, points: Vec<Point>) -> Vec<Rect> {
        self.clear();
        let mut rects = Vec::new();
        if points.len() < 4 {
            unimplemented!("need to push remaining points to rect?");
            return rects;
        } else {
            let mut src = self.new_node(points[0]);
            let mut w = src;

            for i in 1..points.len() {
                let tgt = self.new_node(points[i]);

                if let Some(side) = src.side(tgt) {
                    self.new_edge(src, tgt, side);
                }

                src = tgt;
            }

            unimplemented!()
        }
    }
}
