use std::cmp::Ordering;

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: isize,
    pub y: isize,
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
            Ordering::Equal => match self.x.cmp(&other.x) {
                Ordering::Equal => None,
                o => Some(o),
            },
        }
    }
}

impl Eq for Point {}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.eq(other) {
            Ordering::Equal
        } else {
            self.partial_cmp(other).expect(
                "If points are not equivalent, \
                then they must have some other order.",
            )
        }
    }
}
