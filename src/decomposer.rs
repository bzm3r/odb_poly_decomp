use std::{error::Error, fmt::Display};

use procr_ansi_term::{Color, Style};
use tracing::info;

use crate::active::{ActiveEdges, ActiveNodes, ActiveVec};
use crate::debug::COLOR_ORANGE;
use crate::edge_scans::{EdgeScans, ScanResult};
use crate::emit_info;
use crate::geometry::EdgeTy;
use crate::point::Point;
use crate::rect::Rect;
use crate::{dbg_active_nodes, dbg_decomposer};
use crate::{geometry::Geometry, info_label};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecompErr {
    NotEnoughPoints,
    FailedScanlineUpdate,
    IsAlreadySimple,
}

impl Display for DecompErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for DecompErr {}

#[derive(Clone, Default)]
pub struct Decomposer {
    pub active_nodes: ActiveNodes,
    pub active_edges: ActiveEdges,
    pub scanline: isize,
}

impl Decomposer {
    fn new(geometry: &Geometry) -> Result<Self, DecompErr> {
        let mut active_nodes: ActiveNodes =
            geometry.iter_nodes().map(|(id, _)| id).collect();

        info!(
            "{} active_nodes: {:?}",
            info_label!("pre-sorting"),
            dbg_active_nodes!(geometry, &active_nodes)
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
        //
        // https://en.cppreference.com/w/cpp/algorithm/sort
        // "Sorts the elements in the range [first, last) in non-descending
        // order. The order of equal elements is not guaranteed to be
        // preserved."
        //
        //
        active_nodes.sort(geometry);
        info!(
            "{} active_nodes: {:#?}",
            info_label!("post-sorting"),
            dbg_active_nodes!(geometry, &active_nodes)
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
        info!(
            "{}{}",
            Style::new()
                .fg(Color::Red)
                .bold()
                .blink()
                .paint("SCANLINE: "),
            Style::new()
                .fg(Color::White)
                .bold()
                .paint(format_args!("{}", self.scanline))
        );

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
                "add_edges: node {:?} is on the scanline {:#?} ",
                node, self.scanline
            );
            // Based on: add this node's edges to the active edge list
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L234-L238
            // TODO: Confirm that this is okay/correct (this is not just
            // inserting the active edges vec, but also doing some checks
            // later on to make sure it's inserted in the "right")
            self.active_edges.maybe_insert(geometry, node.inc_edge());
            // let x = geometry[node.inc_edge];
            // self.active_edges.insert_edges(
            //     geometry,
            //     node.inc_edge(),
            //     node.out_edge(),
            // );
        }
    }

    fn scan_and_split(
        &mut self,
        geometry: &mut Geometry,
        mut rects: Vec<Rect>,
    ) -> Vec<Rect> {
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L258-L320
        // See also the comment by CTRL+F for PURGE_ACTIVE_EDGES
        self.active_edges.reset_cursor();

        let mut edge_scan = EdgeScans::default();

        while !self.active_edges.finished() {
            emit_info!(sty:COLOR_ORANGE.bold(),
                fmt:"INITIAL STATE (split loop): {:#?}\n" |
                dbg_decomposer!(self, geometry, Some(&edge_scan))
            );

            match edge_scan.scan_and_split(
                geometry,
                &mut self.active_edges,
                self.scanline,
            ) {
                ScanResult::ReturnRects => {
                    return rects;
                }
                ScanResult::ContinueLoop(s) => {
                    edge_scan = s;
                }
                ScanResult::NewRect(rect) => {
                    emit_info!(
                        fmt:"pushing rect: {:?}" | rect
                    );
                    rects.push(rect);
                }
                ScanResult::ContinueSplit(_) => unreachable!(),
            }
        }
        rects
    }

    #[inline]
    fn update_scanline(&mut self, geometry: &Geometry) {
        self.scanline = self.active_nodes.scanline(geometry).unwrap();
    }

    /// Purge active edge vector.
    #[inline]
    fn purge_active_edges(&mut self, geometry: &Geometry) {
        if self.active_edges.is_empty() {
            return;
        }
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L322-L333
        // The original has to reset the cursor in order to make it point to the
        // begining of the active edges vector again. We do not need to do this,
        // as we can just iterate over the entire vector.
        emit_info!(
            fmt:"state before purging active edges: {:#?}" |
            dbg_decomposer!(self, geometry)
        );
        // We should retain if contains_y returns true, otherwise should purge
        self.active_edges.retain_if(|&id| {
            geometry[id].contains_scanline(geometry, self.scanline)
        });
        emit_info!(
            fmt:"state after purging active edges: {:#?}" |
            dbg_decomposer!(self, geometry)
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
            // Based on (but note: purges must happen *after* the scanline has
            // been updated, we need to update the scanline first,
            // because we do not do the following): https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L205):
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L216
            decomposer.purge_active_edges(&geometry);
            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L208
            decomposer.add_active_edges(&geometry);
            emit_info!(
                fmt:"state after adding active edges: {:#?}" |
                dbg_decomposer!(&decomposer, &geometry, None)
            );

            rects = decomposer.scan_and_split(&mut geometry, rects);

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
}
