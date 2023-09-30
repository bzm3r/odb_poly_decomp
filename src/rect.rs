// Based on:
// https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/include/odb/geom.h#L188

use crate::point::Point;

#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    _left: Point,
    _right: Point,
}

impl Rect {
    pub fn new(left: Point, right: Point) -> Self {
        Self {
            _left: left,
            _right: right,
        }
    }
}
