use crate::node::Node;

#[derive(Clone, Copy, Debug)]
pub enum Side {
    Left,
    Right,
}

// Refs to src/target nodes in Edge are
pub struct Edge<'a> {
    pub src: Box<&'a Node<'a>>,
    pub tgt: Box<&'a Node<'a>>,
    pub side: Side,
}

pub enum InclusionY {
    Strict,
    Weak,
}

impl<'a> Edge<'a> {
    // Based on:
    // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L105
    pub fn test_y_inclusion(&self, y: isize) -> Option<InclusionY> {
        let min = self.src.y().min(self.tgt.y());
        let max = self.src.y().max(self.tgt.y());

        (min <= y && y <= max).then(|| {
            (min < y && y < max)
                .then_some(InclusionY::Weak)
                .unwrap_or(InclusionY::Strict)
        })
    }
}
