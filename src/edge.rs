use crate::node::Node;

#[derive(Clone, Copy, Debug)]
pub enum Side {
    Left,
    Right,
}

// Refs to src/target nodes in Edge are
#[derive(Clone, Copy, Debug)]
pub struct Edge<'a> {
    pub src: &'a Node<'a>,
    pub tgt: &'a Node<'a>,
    pub side: Side,
}

pub enum InclusionY {
    Strict,
    Weak,
}

impl<'a> Edge<'a> {
    pub fn new(src: &'a Node<'a>, tgt: &'a Node<'a>, side: Side) -> Self {
        Edge { src, tgt, side }
    }

    // Based on:
    // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L105
    pub fn test_y_inclusion(&self, y: isize) -> Option<InclusionY> {
        let min = self.src.y().min(self.tgt.y());
        let max = self.src.y().max(self.tgt.y());

        (min <= y && y <= max).then_some({
            if min < y && y < max {
                InclusionY::Weak
            } else {
                InclusionY::Strict
            }
        })
    }
}

// pub struct EdgeIndex(usize);
// pub type Edges<'a> = Vec<Edge<'a>>;

// impl<'a> Index<EdgeIndex> for Edges<'a> {
//     type Output = Edge<'a>;
//     fn index(&self, index: EdgeIndex) -> &Self::Output {
//         &self[index.0]
//     }
// }

// impl<'a> IndexMut<EdgeIndex> for Edges<'a> {
//     fn index_mut(&mut self, index: EdgeIndex) -> &mut Self::Output {
//         &mut self[index.0]
//     }
// }
