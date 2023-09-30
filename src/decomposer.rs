use core::fmt;
use std::{error::Error, fmt::Display};

use nu_ansi_term::Style;
use tracing::info;

use crate::{
    active::Cursor,
    geometry::{Geometry, Side},
};
use crate::{
    active::{ActiveEdges, ActiveNodes, ActiveVec},
    loop_span,
};
use crate::{
    edge::{Edge, EdgeId},
    emit_info,
};
use crate::{misc::debug_with, rect::Rect};
use crate::{
    misc::{MiniStyle, COLOR_ORANGE, LEFT_EDGE, RIGHT_EDGE},
    point::Point,
};

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

#[derive(Clone, Default)]
pub struct Decomposer {
    active_nodes: ActiveNodes,
    active_edges: ActiveEdges,
    scanline: isize,
}

#[derive(Clone, Copy, Debug)]
pub struct EdgeScanResult {
    pub cursor: Cursor,
    pub edge: Edge,
    pub side: Side,
}

impl EdgeScanResult {
    pub fn new(cursor: Cursor, edge: Edge, side: Side) -> Self {
        Self { cursor, edge, side }
    }

    #[inline]
    pub fn scanline_strictly_inside(
        &self,
        geometry: &Geometry,
        scanline: isize,
    ) -> bool {
        self.edge.scanline_strictly_inside(geometry, scanline)
    }

    pub fn matches_id(&self, id: &EdgeId) -> bool {
        &self.edge.id == id
    }

    pub fn matches_cursor(&self, cursor: Cursor) -> bool {
        self.cursor == cursor
    }
}

pub fn debug_active_nodes(
    f: &mut fmt::Formatter,
    geometry: &Geometry,
    active_nodes: &ActiveNodes,
) -> fmt::Result {
    write!(f, "[ ")?;
    for (ix, &id) in active_nodes.items().iter().enumerate() {
        let sep = if ix != 0 { ", " } else { "" };
        let node = geometry[id];

        write!(f, "{sep}{node:?}")?;
    }
    write!(f, " ]")
}

pub fn debug_active_edges(
    f: &mut fmt::Formatter,
    geometry: &Geometry,
    active_edges: &ActiveEdges,
    left: Option<EdgeScanResult>,
    right: Option<EdgeScanResult>,
) -> fmt::Result {
    write!(f, "[ ")?;
    for (ix, &id) in active_edges.items().iter().enumerate() {
        let sep = if ix != 0 { ", " } else { "" };
        let edge = geometry[id];
        let mini = if left.is_some_and(|s| s.matches_id(&id)) {
            LEFT_EDGE
        } else if right.is_some_and(|s| s.matches_id(&id)) {
            RIGHT_EDGE
        } else {
            MiniStyle::default()
        };
        let tag = if left.is_some_and(|s| s.matches_cursor(ix)) {
            "L:"
        } else if right.is_some_and(|s| s.matches_cursor(ix)) {
            "R:"
        } else {
            ""
        };

        write!(
            f,
            "{sep}{:?}",
            Style::from(mini).paint(format!("{tag}{edge:?}"))
        )?;
    }
    write!(f, " ]")
}

impl Decomposer {
    fn new(geometry: &Geometry) -> Result<Self, DecompErr> {
        let mut active_nodes: ActiveNodes =
            geometry.iter_nodes().map(|(id, _)| id).collect();
        info!(
            "(pre-sorting) active_nodes: {:?}",
            debug_with(|f| debug_active_nodes(f, geometry, &active_nodes))
        );

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
            "(post-sorting) active_nodes: {:?}",
            debug_with(|f| debug_active_nodes(f, geometry, &active_nodes))
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
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L258-L320

        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L224
        // See also the comment by CTRL+F for PURGE_ACTIVE_EDGES
        self.active_edges.reset_cursor();
        info!("active_edges_cursor: {:?}", self.active_edges.cursor());

        // Based on: 1) iterate on active_nodes, based on wherever it is
        // currently and 2) if the current does not not lie on the scanline,
        // then stop as we have finished with the nodes that lie on the
        // scanline. https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L226-L232
        while let Some(node) =
            self.active_nodes.next_if(geometry, |geometry, id| {
                let node = geometry[id];
                if node.y() == self.scanline {
                    Some(node)
                } else {
                    // if node.y() != scanline
                    None
                }
            })
        {
            info!(
                "add_edges: {:?} is on the scanline {} ",
                node, self.scanline
            );
            // Based on: add this node's edges to the active edge list
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L234-L238
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

    fn scan_for_edge_on_side(
        &mut self,
        geometry: &Geometry,
        side: Side,
    ) -> Option<EdgeScanResult> {
        // Based on the general shape of:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L268-L276
        // If active_edges.next() returns None (i.e. active_edges' cursor has
        // reached the end), then this function will return `None`, upon which
        // scan edges will also return.

        let fetch_y = match side {
            Side::Left => Edge::src_y,
            Side::Right => Edge::tgt_y,
        };

        while let Some(edge) = self.active_edges.next(geometry) {
            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L273-L276
            if edge.side == side && fetch_y(&edge, geometry) != self.scanline {
                return Some(EdgeScanResult::new(
                    self.active_edges.cursor(),
                    edge,
                    side,
                ));
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
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L258-L320
        // See also the comment by CTRL+F for PURGE_ACTIVE_EDGES
        self.active_edges.reset_cursor();

        let mut opt_left;
        let mut opt_right = None;

        while !self.active_edges.finished() {
            let _ = loop_span!(
                sty:COLOR_ORANGE, id:"scan_edges: while !self.active_edges.finished()"
            );

            opt_left = None;
            emit_info!(
                fmt:"initial state: {:?}\n" |
                debug_with(|f| self.debug(f, geometry, opt_left, opt_right))
            );

            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L265-L277
            opt_left = self.scan_for_edge_on_side(geometry, Side::Left);
            let mut left = if let Some(left) = opt_left {
                left
            } else {
                // TODO: should we be panicking here?
                emit_info!(sty:COLOR_ORANGE, msg:"no left edge found. finishing.");
                return rects;
            };

            emit_info!(
                fmt:"after scanning for left edge: {:?}\n" |
                debug_with(|f| self.debug(f, geometry, opt_left, opt_right))
            );

            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L279-L290
            // NOTE: we do not have to do:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L281
            // --- because our "iterator" increments itself each time we call
            // next.
            opt_right = self.scan_for_edge_on_side(geometry, Side::Right);
            let mut right = if let Some(right) = opt_right {
                right
            } else {
                // TODO: should we be panicking here?
                emit_info!(sty:COLOR_ORANGE, msg:"no right edge found. finishing.");
                return rects;
            };

            emit_info!(
                fmt:"after scanning for right edge: {:?}\n" |
                debug_with(|f| self.debug(f, geometry, opt_left, opt_right))
            );

            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L293
            if left.scanline_strictly_inside(geometry, self.scanline)
                && right.scanline_strictly_inside(geometry, self.scanline)
            {
                // Current interpretation of (++itr) == right.cursor is that
                // itr is incremented first, and then the comparison takes
                // place. https://stackoverflow.com/a/1813008/3486684
                left.cursor += 1;
                if left.cursor == right.cursor {
                    continue;
                } else {
                    if let Some(id) = self.active_edges.peek_at(left.cursor) {
                        left.edge = geometry[id];
                    } else {
                        // TODO: maybe we want to return with rects here?
                        continue;
                    }
                    // Need to update left if we have reached here
                    opt_left.replace(left);
                }
            }
            emit_info!(
                fmt:"after confirming check if both left and right strictly contain scanline: {:?}\n" |
                debug_with(|f| self.debug(f, geometry, opt_left, opt_right))
            );

            left.edge = if left
                .scanline_strictly_inside(geometry, self.scanline)
            {
                // Based on:
                // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L299-L303
                let new_edge =
                    self.split_edge(geometry, left.edge.id, Side::Left);
                // Based on:
                // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L304
                geometry[left.edge.id] = new_edge;
                opt_left.replace(EdgeScanResult {
                    cursor: left.cursor,
                    edge: new_edge,
                    side: left.side,
                });
                emit_info!(
                    fmt:"left strictly contains scanline, so performed a split: {:?}\n" |
                    debug_with(|f| self.debug(f, geometry, opt_left, opt_right))
                );
                new_edge
            } else {
                left.edge
            };

            right.edge = if right
                .scanline_strictly_inside(geometry, self.scanline)
            {
                // Based on:
                // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L309-L313
                let new_edge =
                    self.split_edge(geometry, right.edge.id, Side::Right);
                // Based on:
                // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L314
                geometry[right.edge.id] = new_edge;
                opt_right.replace(EdgeScanResult {
                    cursor: right.cursor,
                    edge: new_edge,
                    side: right.side,
                });
                emit_info!(
                    fmt:"right strictly contains scanline, so performed a split: {:?}" |
                    debug_with(|f| self.debug(f, geometry, opt_left, opt_right))
                );
                new_edge
            } else {
                right.edge
            };

            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L317
            let rect = Rect::new(
                left.edge.source(geometry).point,
                right.edge.source(geometry).point,
            );
            emit_info!(
                fmt:"pushing rect: {:?}" | rect
            );
            rects.push(rect);
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
        input_edge_id: EdgeId,
        side: Side,
    ) -> Edge {
        // "split intersected edge"
        // Based on (for left edge):
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L299-304
        // Based on (for right  edge): https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#309-314

        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L299
        // and  https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L309
        // existing corresponds to u (left)/w (right) in the original code
        let input_node_id = match side {
            Side::Left => geometry[input_edge_id].source,
            Side::Right => geometry[input_edge_id].target,
        };
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L300
        //  https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L310
        let existing_x = geometry[input_node_id].x();

        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L300
        //  https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L310
        // TODO: confirm that the edge should not be added to active nodes list
        let new_node_id = geometry.new_node(
            Point::new(existing_x, self.scanline),
            None,
            None,
        );

        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L301-L303
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L311-L314
        match side {
            Side::Left => {
                // input edge gets its source replaced by new node
                geometry[input_edge_id].set_source(new_node_id);
                // input node (source of input edge) has its outgoing edge (the
                // input edge) deleted
                geometry[input_node_id].take_out_edge();
                // new node has its outgoing edge set to the input_edge
                geometry[new_node_id].set_out_edge(input_edge_id);
                // insert a new edge into the geometry edge list
                // TODO: (Note: we do not add it to the active edge list, since
                // we do not want to split it? No: it's because we will call
                // add_edges again later, that should manage adding in new edges
                // into the active edge list, if it still needs to be split.)
                geometry.new_edge(input_node_id, new_node_id, side)
            }
            Side::Right => {
                // input edge gets its target replaced by new node
                geometry[input_edge_id].set_target(new_node_id);
                // input node (target of input edge) has its incoming edge
                // (the input edge) deleted
                geometry[input_node_id].take_inc_edge();
                // new node has its incoming edge set to be the input edge
                geometry[new_node_id].set_inc_edge(input_edge_id);
                // TODO: (Note: we do not add it to the active edge list, since
                // we do not want to split it? No: it's because we will call
                // add_edges again later, that should manage adding in new edges
                // into the active edge list, if it still needs to be split.)
                geometry.new_edge(new_node_id, input_node_id, side)
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
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L322-L333
        // The original has to reset the cursor in order to make it point to the
        // begining of the active edges vector again. We do not need to do this,
        // as we can just iterate over the entire vector.
        emit_info!(
            fmt:"state before purging active edges: {:?}" |
            debug_with(|f| self.debug(f, geometry, None, None))
        );
        // We should retain if contains_y returns true, otherwise should purge
        self.active_edges.retain_if(|&id| {
            geometry[id].contains_scanline(geometry, self.scanline)
        });
        emit_info!(
            fmt:"state after purging active edges: {:?}" |
            debug_with(|f| self.debug(f, geometry, None, None))
        );
        // TODO: PURGE_ACTIVE_EDGES
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
            // Based on (but purges must happen *after* the scanline has been
            // updated, and in our case, we need to update the scanline first,
            // because we do not do: https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L205):
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L216
            decomposer.purge_active_edges(&geometry);
            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L208
            decomposer.add_active_edges(&geometry);
            emit_info!(
                fmt:"state after adding active edges: {:?}" |
                debug_with(|f| decomposer.debug(f, &geometry, None, None))
            );

            rects = decomposer.scan_edges(&mut geometry, rects);
            emit_info!(
                fmt:"state after scanning edges: {:?}" |
                debug_with(|f| decomposer.debug(f, &geometry, None, None))
            );

            if decomposer.active_nodes.finished() {
                break;
            }
            // TODO: do we need something that does what line 214 does?:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L214
            // Answer: I don't think so, because it's manually advancing the
            // iterator pointer, which we do not need to do? However, we
            // should make sure that updating of the scanline happens first
            // in the loop (SCANLINE_COMMENT)
        }
        Ok(rects)
    }

    pub fn debug(
        &self,
        f: &mut fmt::Formatter,
        geometry: &Geometry,
        left: Option<EdgeScanResult>,
        right: Option<EdgeScanResult>,
    ) -> fmt::Result {
        writeln!(f, "Decomposer {{")?;
        debug_with(|f| debug_active_nodes(f, geometry, &self.active_nodes));
        writeln!(f)?;
        debug_with(|f| {
            debug_active_edges(f, geometry, &self.active_edges, left, right)
        });
        writeln!(f)?;
        writeln!(f, "\tscanline: {},", self.scanline)?;
        writeln!(f, "}}")
    }
}
