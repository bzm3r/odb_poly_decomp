use std::{cmp::Ordering, fmt::Debug};

use crate::geometry::Side;

#[derive(Clone, Copy, Default)]
pub struct Point {
    pub x: isize,
    pub y: isize,
}

impl Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Point({},{})", self.x, self.y)
    }
}

impl Point {
    #[inline]
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn which_side(&self, other: &Point) -> Option<Side> {
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L188

        // Not really sure why it's called "Left/Right" when it's more "Up/Down"
        // Should probably rename it. Then again, it doesn't really matter; one
        // person's up is just another person's Left.
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

// We are purposefully overriding partial_cmp
#[allow(clippy::incorrect_partial_ord_impl_on_ord_type)]
impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // POINT_PARTIAL_CMP
        // Usage is based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L203
        //
        // Note that the NodeCompare function returns true or false. This is
        // what the `std::sort` definition in cppref says:
        // https://en.cppreference.com/w/cpp/algorithm/sort
        //
        // A sequence is sorted with respect to a comparator `comp` if for any
        // iterator `it` pointing to the sequence and any non-negative integer
        // `n` such that `it + n` is a valid iterator pointing to an
        // element of the sequence, `comp(*(it + n), *it)` `(or *(it + n) <
        // *it)` evaluates to `false`.
        //
        // Deciphering this arcane bullshit closely (seriously, what was math
        // notation invented for if not to express exactly stuff like this with
        // care?).
        //
        // For any `0 <= k < n < vec.len()`, comp(vec[n], vec[k]) is false.
        // Put differently. (vec[n] < vec[k]) is false. In other words, we are
        // ordering elements in a "non-decreasing" (i.e. increasing or equal)
        // order See now the definition comment below.

        // Definition is based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L71
        // Okay, so we see here that the NodeCompare returns if self.y <
        // other.y, Otherwise, it returns false. So, we should be ordering
        // elements by least to greatest (in terms of y). Check
        // sorted_active_nodes trace to check that this is happening.
        match self.y.cmp(&other.y) {
            o @ (Ordering::Less | Ordering::Greater) => Some(o),
            // When ys are equal, we sort by xs, again in non-decreasing order.
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
