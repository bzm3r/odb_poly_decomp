use procr_ansi_term::{Color, Style};
use tracing::info;

use crate::{
    active::{ActiveEdges, ActiveVec, Cursor},
    dbg_active_edges,
    debug::COLOR_ORANGE,
    edge::{Edge, EdgeId},
    emit_info,
    geometry::{EdgeTy, Geometry},
    rect::Rect,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct EdgeScans {
    le: Option<Edge>,
    re: Option<Edge>,
    lc: Option<Cursor>,
    rc: Option<Cursor>,
}

#[derive(Clone, Copy, Debug)]
pub enum ScanResult {
    ReturnRects,
    ContinueLoop(EdgeScans),
    ContinueSplit(EdgeScans),
    NewRect(Rect),
}

macro_rules! check_return {
    ($msg:literal, $geometry:expr, $active_edges:expr, $self:expr, $operation:expr) => {
        {
            let result = $operation;
            emit_info!(sty:Color::Blue,
                fmt:"{}: {:#?}\n" | $msg,
                dbg_active_edges!($geometry, $active_edges, &$self)
            );
            info!("{:#?}", &$self);
            match result {
                ScanResult::ContinueSplit(s) => {
                    info!("{}", COLOR_ORANGE.paint("continuing split..."));
                    s
                },
                r => {
                    emit_info!(sty:COLOR_ORANGE, fmt:"returning {:?}..." | r);
                    return r;
                }
            }
        }
    };
}

impl EdgeScans {
    pub fn matches_edge(&self, id: EdgeId) -> Option<EdgeTy> {
        if let Some(le) = self.le {
            if le.id() == id {
                return Some(le.ty);
            }
        }

        if let Some(re) = self.re {
            if re.id() == id {
                return Some(re.ty);
            }
        }
        None
    }

    pub fn matches_cursor(&self, cursor: Cursor) -> Option<EdgeTy> {
        if let Some(lc) = self.lc {
            if lc == cursor {
                return Some(EdgeTy::Left);
            }
        }

        if let Some(rc) = self.rc {
            if rc == cursor {
                return Some(EdgeTy::Right);
            }
        }
        None
    }

    pub fn scan_for_edges(
        mut self,
        active_edges: &mut ActiveEdges,
        scanline: isize,
        geometry: &Geometry,
    ) -> ScanResult {
        // Based on the general shape of:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L267-L276
        // If active_edges.next() returns None (i.e. active_edges' cursor has
        // reached the end), then this function will return `None`, upon which
        // scan edges will also return.

        // This essentially finds the left edge with the largest index in the
        // active edges list.
        while let Some(edge) = active_edges.next(geometry) {
            info!(
                "setting as LE: active edge: {}",
                Style::new()
                    .fg(Color::Purple)
                    .bold()
                    .paint(format_args!("{}", edge.id().index()))
            );
            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L273-L276
            self.le.replace(edge);
            if edge.ty == EdgeTy::Left
                && Edge::src_y(&edge, geometry) != scanline
            {
                self.lc.replace(active_edges.cursor());
                break;
            }
        }

        if active_edges.finished() {
            return ScanResult::ReturnRects;
        }

        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L279-L290
        // NOTE: we do not have to do the extra initialization increment
        // seen in the for loop here: https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L281
        // --- because our iterator increments itself each time we call
        // next.
        while let Some(edge) = active_edges.next(geometry) {
            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L273-L276
            self.re.replace(edge);
            if edge.ty == EdgeTy::Right
                && Edge::tgt_y(&edge, geometry) != scanline
            {
                self.rc.replace(active_edges.cursor());
                break;
            }
        }

        ScanResult::ContinueSplit(self)
    }

    #[inline]
    pub fn le(&self) -> &Edge {
        self.le.as_ref().expect(
            "expect to have found some left edge if calling `le` method",
        )
    }

    #[inline]
    pub fn re(&self) -> &Edge {
        self.re.as_ref().expect(
            "expect to have found some right edge if calling `re` method",
        )
    }

    fn continue_split(self) -> ScanResult {
        ScanResult::ContinueSplit(self)
    }

    fn continue_loop(self) -> ScanResult {
        ScanResult::ContinueLoop(self)
    }

    fn check_both_splittable(
        mut self,
        geometry: &Geometry,
        active_edges: &ActiveEdges,
        scanline: isize,
    ) -> ScanResult {
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L293
        if self.le().scanline_strictly_inside(geometry, scanline)
            && self.re().scanline_strictly_inside(geometry, scanline)
        {
            // Current interpretation of (++itr) == right_iter is that
            // itr is incremented first, and then the comparison takes
            // place. https://stackoverflow.com/a/1813008/3486684
            //
            // So this is testing to see if the scanline is strictly inside the
            // edges, and the next one is
            if let Some(c) = self.lc.as_mut() {
                *c += 1;
            }
            if self.lc == self.rc {
                return self.continue_loop();
            } else if let Some(id) =
                self.lc.and_then(|c| active_edges.peek_at(c))
            {
                self.le.replace(geometry[id]);
            }
        }
        self.continue_split()
    }

    pub fn scan_and_split(
        mut self,
        geometry: &mut Geometry,
        active_edges: &mut ActiveEdges,
        scanline: isize,
    ) -> ScanResult {
        // Based on:
        // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L265-L277
        self = check_return!(
            "after scanning for edges",
            geometry,
            active_edges,
            self,
            self.scan_for_edges(active_edges, scanline, geometry)
        );

        self = check_return!(
            "after checking if both are splittable",
            geometry,
            active_edges,
            self,
            self.check_both_splittable(geometry, active_edges, scanline)
        );

        if self.le().scanline_strictly_inside(geometry, scanline) {
            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L299-L303
            let new_edge = geometry.split_edge(self.le().id(), scanline);
            self.le.replace(new_edge);
            emit_info!(
                fmt:"left strictly contains scanline, so performed a split: {:#?}\n" |
                dbg_active_edges!(geometry, active_edges, &self)
            );
        } else if self.re().scanline_strictly_inside(geometry, scanline) {
            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L299-L303
            let new_edge = geometry.split_edge(self.re().id(), scanline);
            self.re.replace(new_edge);
            emit_info!(
                fmt:"right strictly contains scanline, so performed a split: {:#?}\n" |
                dbg_active_edges!(geometry, active_edges, &self)
            );
        };

        ScanResult::NewRect(
            // Based on:
            // https://github.com/bzm3r/OpenROAD/blob/ecc03c290346823a66fec78669dacc8a85aabb05/src/odb/src/zutil/poly_decomp.cpp#L317
            Rect::new(
                self.le().source(geometry).point,
                self.re().source(geometry).point,
            ),
        )
    }
}
