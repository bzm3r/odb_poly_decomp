use std::{error::Error, fmt::Display};

use tracing::info;

use crate::active::{ActiveEdges, ActiveNodes, ActiveVec, Cursor};
use crate::edge::{Edge, EdgeId};
use crate::geometry::{Geometry, Side};
use crate::point::Point;
use crate::rect::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecompErr {
    NotEnoughPoints,
    FailedScanlineUpdate,
}

impl Display for DecompErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for DecompErr {}

#[derive(Clone, Debug, Default)]
pub struct Decomposer {
    active_nodes: ActiveNodes,
    active_edges: ActiveEdges,
    scanline: isize,
}

impl Decomposer {
    fn new(geometry: &Geometry) -> Result<Self, DecompErr> {
        let mut active_nodes: ActiveNodes =
            geometry.iter_nodes().map(|(id, _)| id).collect();
        info!("active_nodes: {:?}", &active_nodes);

        let active_edges = ActiveEdges::with_capacity(2 * active_nodes.len());

        // We do not need to do scanline update here, as we do it as part of the
        // loop decomposition loop. (CTRL+F for "DECOMP_SCANLINE_UPDATE" below)
        // let scanline = active_nodes
        //     .scanline(geometry)
        //     .ok_or(DecompErr::FailedScanlineUpdate)?;

        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#203
        // Follow the sort -> cmp chain all the way to the definition of
        // PartialCmp for Point (in point.rs), or search all files for:
        // POINT_PARTIAL_CMP
        active_nodes.sort(geometry);
        info!(
            "sorted active nodes: {:?}",
            active_nodes
                .items()
                .iter()
                .map(|&node| geometry[node].point)
                .collect::<Vec<Point>>()
        );

        Ok(Self {
            active_nodes,
            active_edges,
            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#205
            scanline: 0,
        })
    }

    // Based on: it is called add_edges in the original, but this is a misnomer
    // as it is specifically adding to the active edges. We only add to edges
    // during the scan_edges phase.
    // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L222
    fn add_active_edges(&mut self, geometry: &Geometry) {
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L258-320
        // See also the comment by CTRL+F for PURGE_ACTIVE_EDGES
        self.active_edges.reset_cursor();
        info!("active_edges_cursor: {:?}", self.active_edges.cursor());
        // Based on: 1) iterate on active_nodes, based on wherever it is
        // currently and 2) if the current node's y-marker != scanline, then
        // we should stop, as we have finished with the set of edges relevant
        // for this scanline
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L226-232
        while let Some(node) =
            self.active_nodes.next_if(geometry, |geometry, id| {
                let node = geometry[id];
                if node.y() == self.scanline {
                    Some(node)
                } else {
                    None
                }
            })
        {
            // Based on: add this node's edges to the active edge list
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L234-238
            // TODO: Confirm that this is okay/correct (this is not just
            // inserting into the active edges vec, but also doing some checks
            // later on down the road)
            self.active_edges.insert_edges(
                geometry,
                node.inc_edge(),
                node.out_edge(),
            );
        }
    }

    fn scan_side_edge(
        &mut self,
        geometry: &Geometry,
        required: Side,
    ) -> Option<(Edge, Cursor)> {
        // Based on the general shape of:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L268-277

        // If active_edges.next() returns None (i.e. active_edges' cursor has
        // reached the end), then this function will return `None`, which will
        // cause scan edges to return as well through use of the `?` operator.
        while let Some(edge) = self.active_edges.next(geometry) {
            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L273-276
            if edge.side == required && edge.src_y(geometry) != self.scanline {
                return Some((edge, self.active_edges.cursor()));
            }
        }
        None
    }

    fn scan_edges(
        &mut self,
        geometry: &mut Geometry,
        mut rects: Vec<Rect>,
    ) -> Vec<Rect> {
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L258-320
        // See also the comment by CTRL+F for PURGE_ACTIVE_EDGES
        self.active_edges.reset_cursor();

        let mut left_cursor;
        let mut right_cursor;
        let mut left;
        let mut right;

        while self.active_edges.finished() {
            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L265-277
            (left, left_cursor) = if let Some((e, c)) =
                self.scan_side_edge(geometry, Side::Left)
            {
                (e, c)
            } else {
                // TODO: should we be panicking here?
                return rects;
            };

            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L279-290
            (right, right_cursor) = if let Some((e, c)) =
                self.scan_side_edge(geometry, Side::Right)
            {
                (e, c)
            } else {
                // TODO: should we be panicking here?
                return rects;
            };

            if left.inside_y(geometry, self.scanline)
                && right.inside_y(geometry, self.scanline)
            {
                // https://stackoverflow.com/a/1813008/3486684 sigh, C++
                left_cursor += 1;
                if left_cursor == right_cursor {
                    continue;
                }
            }

            if left.inside_y(geometry, self.scanline) {
                // The existing edge is not deleted, so its sufficient to do
                // an overwrite of the variable that used to contain it?
                left = self.split_edge(geometry, left.id, Side::Left);
            }

            if right.inside_y(geometry, self.scanline) {
                right = self.split_edge(geometry, right.id, Side::Right);
            }

            rects.push(Rect::new(
                left.source(geometry).point,
                right.source(geometry).point,
            ));
        }
        rects
    }

    // This is way too symmetric to not be simplified. Idea should be:
    // edge methods should take `side` in order to return source/target or
    // set source/target appropriately. (Essentially, must view an edges
    // end points not only as source/target, but *just* as endpoints, which
    // are then split).
    fn split_edge(
        &mut self,
        geometry: &mut Geometry,
        existing_edge_id: EdgeId,
        side: Side,
    ) -> Edge {
        // "split intersected edge"
        let existing_node_id = match side {
            Side::Left => geometry[existing_edge_id].source,
            Side::Right => geometry[existing_edge_id].target,
        };
        let existing_x = geometry[existing_node_id].x();

        // Based on: https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L300
        // confirm that the edge should not be added to active nodes list
        let new_node_id = geometry.new_node(
            Point::new(existing_x, self.scanline),
            None,
            None,
        );

        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L301-303
        match side {
            Side::Left => {
                geometry[existing_edge_id].set_source(new_node_id);
                geometry[existing_node_id].take_out_edge();
                geometry[new_node_id].set_out_edge(existing_edge_id);
                geometry.new_edge(existing_node_id, new_node_id, side)
            }
            Side::Right => {
                geometry[existing_edge_id].set_source(new_node_id);
                geometry[existing_node_id].take_in_edge();
                geometry[new_node_id].set_inc_edge(existing_edge_id);
                geometry.new_edge(new_node_id, existing_node_id, side)
            }
        }
    }

    #[inline]
    fn update_scanline(&mut self, geometry: &Geometry) {
        self.scanline = self.active_nodes.scanline(geometry).unwrap();
    }

    /// Purge active edge vector.
    #[inline]
    fn purge_active_edges(&mut self, geometry: &Geometry) {
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L322-333
        self.active_edges
            .retain_if(|&id| geometry[id].contains_y(geometry, self.scanline));
        info!("post-purge cursor: {}", self.active_edges.cursor());
        // PURGE_ACTIVE_EDGES
        // Now that we have purged this edge iterator, its cursor is invalid.
        // The original code is set up so that `add_edges` is called next
        // if the scan/decompose loop still runs. The first thing `add_edges`
        // does is set the cursor of active edges to 0...
        // TODO: Maybe we should be putting cursor in an `Option`, and then
        // setting it to none as a consequence of this operation?
    }

    /// Initialized with the vertical edges needed for scanline intersection
    /// test.
    ///
    /// It requires that the supplied polygon points are sorted in clockwise
    /// order.
    ///
    /// Based on:
    /// https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L183
    pub fn decompose(points: Vec<Point>) -> Result<Vec<Rect>, DecompErr> {
        let mut geometry = Geometry::new(points)?;
        let mut decomposer = Self::new(&geometry)?;

        // TODO: figure out whether its worth pre-allocating rects. If yes, then
        // what value should we pick? Currently just picked 2 * n_points...
        //
        // Note however that n_points is just the initial number of points
        // forming the rectilinear polygon. New nodes might be added. How many
        // more? Suppose each edge sees one split, then we will end up having
        // 2 * n_points, but is this an absolute upper bound? Can there be an
        // edge which is split more than once? Yes, so this is not an upper
        // bound, but just an initial guess.
        //
        // How many rects then? An absolute upper bound for the number of
        // rectangles that can exist in the decomposition is 4 * n_nodes. This
        // is because each node can only be the corner of 4 different rects.
        //
        // However in our actual scenario, all our nodes are *border* nodes.
        // This suggests that each node can have at most 3 rects. So, we get: 3
        // * n_nodes = (6 * n_points) rects
        //
        // Worth pre-allocating? Not sure.
        let mut rects = Vec::with_capacity(geometry.len_nodes());
        loop {
            // Based on (see also, by CTRL+F for "SCANLINE_COMMENT" below):
            // DECOMP_SCANLINE_UPDATE https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L215
            decomposer.update_scanline(&geometry);
            info!("updated scanline: {:?}", decomposer.scanline);
            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L208
            decomposer.add_active_edges(&geometry);
            info!("active_edges: {:?}", &decomposer.active_edges);

            rects = decomposer.scan_edges(&mut geometry, rects);

            if decomposer.active_nodes.finished() {
                break;
            } else {
                // TODO: do we need something that does what line 214 does?:
                // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L214
                // Answer: I don't think so, because it's manually advancing the
                // iterator pointer, which we do not need to do? However, we
                // should make sure that updating of the scanline happens first
                // in the loop (SCANLINE_COMMENT)

                // Based on:
                // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L216
                decomposer.purge_active_edges(&geometry);
            }
        }
        Ok(rects)
    }
}
