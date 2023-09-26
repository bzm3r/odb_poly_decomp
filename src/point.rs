use std::cmp::Ordering;

use crate::geometry::Side;

#[derive(Clone, Copy, Debug, Default)]
pub struct Point {
    pub x: isize,
    pub y: isize,
}

impl Point {
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    // Based on:
    // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L188
    pub fn which_side(&self, other: &Point) -> Option<Side> {
        match self.y.cmp(&other.y) {
            Ordering::Less => Some(Side::Left),
            Ordering::Equal => None,
            Ordering::Greater => Some(Side::Right),
        }
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x.eq(&other.x) && self.y.eq(&other.y)
    }
}

impl PartialOrd for Point {
    // Based on:
    // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L71
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.y.cmp(&other.y) {
            o @ (Ordering::Less | Ordering::Greater) => Some(o),
            Ordering::Equal => Some(self.x.cmp(&other.x)),
        }
    }
}

impl Eq for Point {}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect(
            "Points have a total order, so partial_cmp \
            should not return None.",
        )
    }
}
