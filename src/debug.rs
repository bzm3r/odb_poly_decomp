use id_arena::Arena;
use itertools::Itertools;
use procr_ansi_term::{Color, Style};
use std::fmt;

use crate::{
    active::ActiveEdges,
    active::{ActiveNodes, ActiveVec},
    decomposer::Decomposer,
    edge::{Edge, EdgeId},
    edge_scans::EdgeScans,
    geometry::{EdgeTy, Geometry},
    node::Node,
    point::Point,
};

const STYLE_TYPE_NAME: Style = Style::new().bold().fg(Color::Purple);
const STYLE_TYPE_ID: Style = Style::new().fg(Color::White).bold();
const STYLE_LABEL: Style = Style::new().fg(Color::Yellow);
const STYLE_ITEM: Style = Style::new().fg(Color::Cyan);

pub const COLOR_GREEN: Color = Color::Fixed(40);
pub const COLOR_BLUE: Color = Color::Fixed(27);
pub const COLOR_ORANGE: Color = Color::Fixed(208);

pub const STYLE_CURSOR: Style =
    Style::new().underline().bold().fg(Color::White);

pub const LEFT_SCAN_EDGE: Style = Style::new().fg(COLOR_GREEN);

pub const RIGHT_SCAN_EDGE: Style = Style::new().fg(COLOR_BLUE);

pub fn debug_with(
    f: impl Fn(&mut fmt::Formatter) -> fmt::Result,
) -> impl fmt::Debug {
    struct DebugWith<F>(F);

    impl<F> fmt::Debug for DebugWith<F>
    where
        F: Fn(&mut fmt::Formatter) -> fmt::Result,
    {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            self.0(f)
        }
    }

    DebugWith(f)
}

#[macro_export]
macro_rules! type_name {
    ($name:expr) => {
        STYLE_TYPE_NAME.paint($name)
    };
}

#[macro_export]
macro_rules! type_id {
    ($label:expr, $id:expr) => {
        format!("{}{}", STYLE_LABEL.paint($label), type_id!(@id_expr $id))
    };
    ($id:expr) => {
        type_id!(@id_expr $id)
    };
    (@id_expr $id:expr) => {
        STYLE_TYPE_ID
            .paint::<fmt::Arguments, str>(format_args!("[{}]", $id.index())).to_string()
    };
}

#[macro_export]
macro_rules! item {
    ($label:literal, $item:expr) => {
        format_args!("{}:{}", STYLE_LABEL.paint($label), $item)
    };
}

#[macro_export]
macro_rules! item_dbg {
    ($label:expr, $item:expr) => {
        format!("{}:{:?}", STYLE_LABEL.paint($label), $item)
    };
    ($item:expr) => {
        format!("{}", STYLE_ITEM.paint(format_args!("{:?}", $item)))
    };
}

impl fmt::Debug for EdgeTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EdgeTy::Left => "L",
                EdgeTy::Right => "R",
            }
        )
    }
}

#[macro_export]
macro_rules! dbg_edge {
    ($geometry:expr, $edge:expr) => {
        $crate::debug::debug_with($crate::debug::debug_edge($geometry, $edge))
    };
}

pub fn debug_edge<'a>(
    geometry: &'a Geometry,
    edge: &'a Edge,
) -> impl Fn(&mut fmt::Formatter) -> fmt::Result + 'a {
    move |f| {
        write!(f, "{}", Style::new().reset_prefix().paint(""))?;
        write!(
            f,
            "{}",
            STYLE_LABEL.paint(format!("{:?}E{}:", edge.ty, edge.id.index()))
        )?;
        write!(
            f,
            "\n    {}\n  ->{}",
            item_dbg!(debug_with(debug_node_endpoint(&geometry[edge.source]))),
            item_dbg!(debug_with(debug_node_endpoint(&geometry[edge.target]))),
        )
    }
}

#[macro_export]
macro_rules! dbg_edges {
    ($geometry:expr, $edges:expr) => {
        $crate::debug::debug_with($crate::debug::debug_edges($geometry, $edges))
    };
}

pub fn debug_edges<'a>(
    geometry: &'a Geometry,
    edges: &'a Arena<Edge>,
) -> impl Fn(&mut fmt::Formatter) -> fmt::Result + 'a {
    |f| {
        writeln!(f)?;
        for (_, edge) in edges.iter() {
            writeln!(f, "{:?}", debug_with(debug_edge(geometry, edge)))?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! dbg_edge_incident {
    ($geometry:expr, $edge:expr, $edge_type:expr) => {
        $crate::debug::debug_with($crate::debug::debug_edge_incident(
            $geometry, $edge, $edge_type,
        ))
    };
}

enum IncidentEdgeType {
    Incoming,
    Outgoing,
}

fn debug_edge_incident(
    geometry: &Geometry,
    edge_id: Option<EdgeId>,
    edge_type: IncidentEdgeType,
) -> impl Fn(&mut fmt::Formatter) -> fmt::Result + '_ {
    move |f| {
        write!(f, "{}", Style::new().reset_prefix().paint(""))?;
        if let Some(edge) = edge_id.map(|e| &geometry[e]) {
            match edge_type {
                IncidentEdgeType::Incoming => {
                    write!(
                        f,
                        "{}",
                        STYLE_LABEL.paint(format!(
                            "{:?}E{}:",
                            edge.ty,
                            edge.id.index()
                        ))
                    )?;
                    write!(
                        f,
                        "{} ->",
                        Style::new().fg(Color::LightRed).paint(format_args!(
                            "{}",
                            &geometry.nodes[edge.source].point
                        ))
                    )
                }
                IncidentEdgeType::Outgoing => {
                    write!(
                        f,
                        "-> {}",
                        Style::new().fg(Color::LightRed).paint(format_args!(
                            "{}",
                            &geometry.nodes[edge.target].point
                        ))
                    )?;
                    write!(
                        f,
                        "{}",
                        STYLE_LABEL.paint(format!(
                            ":{:?}E{}",
                            edge.ty,
                            edge.id.index()
                        ))
                    )
                }
            }
        } else {
            match edge_type {
                IncidentEdgeType::Incoming => f.write_str("_ ->"),
                IncidentEdgeType::Outgoing => f.write_str("-> _"),
            }
        }
    }
}

impl fmt::Debug for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", type_name!("E"))?;
        write!(f, "{}", type_id!(self.id))?;
        write!(
            f,
            "{:?}({}--{})",
            self.ty,
            type_id!("s", self.source),
            type_id!("t", self.target),
        )
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("@({},{})", self.x, self.y))
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[macro_export]
macro_rules! dbg_node {
    ($geometry:expr, $node:expr) => {
        $crate::debug::debug_with($crate::debug::debug_node($geometry, $node))
    };
}

pub fn debug_node<'a>(
    geometry: &'a Geometry,
    node: &'a Node,
) -> impl Fn(&mut fmt::Formatter) -> fmt::Result + 'a {
    move |f| {
        write!(f, "{}", Style::new().reset_prefix().paint(""))?;
        write!(
            f,
            "{}:",
            STYLE_LABEL.bold().paint(format!("N{}", node.id.index()))
        )?;
        write!(
            f,
            "  {:?}  {}  {:?}",
            dbg_edge_incident!(
                geometry,
                node.inc_edge,
                IncidentEdgeType::Incoming
            ),
            Style::new()
                .bold()
                .underline()
                .fg(Color::LightCyan)
                .paint(format_args!("{:?}", node.point)),
            dbg_edge_incident!(
                geometry,
                node.out_edge,
                IncidentEdgeType::Outgoing
            )
        )
    }
}

pub fn debug_node_endpoint(
    node: &Node,
) -> impl Fn(&mut fmt::Formatter) -> fmt::Result + '_ {
    move |f| {
        write!(f, "{}", Style::new().reset_prefix().paint(""))?;
        write!(
            f,
            "{}{}",
            STYLE_LABEL.paint(format_args!("N{}:", node.id.index())),
            format_args!(
                " {} -> {} ->{}",
                node.inc_edge
                    .map(|e| e.index().to_string())
                    .unwrap_or("_".to_string()),
                STYLE_ITEM
                    .underline()
                    .paint(format_args!("{:?}", node.point)),
                node.out_edge
                    .map(|e| e.index().to_string())
                    .unwrap_or("_".to_string())
            )
        )
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug_node_endpoint(self)(f)
    }
}

#[macro_export]
macro_rules! info_label {
    ($label:literal) => {
        Style::new()
            .bold()
            .fg(Color::Yellow)
            .paint(format_args!("({})", $label))
    };
}

#[macro_export]
macro_rules! loop_span {
    (sty:$style:ident, id:$loop_id:literal) => {
        tracing::info_span!($loop_id, "{}", $style.paint("=============="))
    };
}

#[macro_export]
macro_rules! emit_info {
    (sty:$style:expr, msg:$msg:literal) => {
        tracing::info!("{}", $style.paint($msg));
    };
    (sty:$style:expr, fmt:$fmt:literal | $($rest:tt)+) => {
        tracing::info!("{}", $style.paint(format_args!($fmt, $($rest)*)));
    };
    (fmt:$fmt:literal | $($rest:tt)+) => {
        tracing::info!($fmt, $($rest)*);
    };
}

#[macro_export]
macro_rules! emit_info_span {
    (sty:$style:ident, msg:$msg:literal) => {
        tracing::info_span!("{}", $style.paint($msg));
    };
    (sty:$style:ident, fmt:$fmt:literal | $($rest:tt)+) => {
        tracing::info_span!("{}", $style.paint(format_args!($fmt, $($rest)*)));
    };
    (fmt:$fmt:literal | $($rest:tt)+) => {
        tracing::info_span!($fmt, $($rest)*);
    };
}

pub fn _display_with(
    f: impl Fn(&mut fmt::Formatter) -> fmt::Result,
) -> impl fmt::Display {
    struct DisplayWith<F>(F);

    impl<F> fmt::Display for DisplayWith<F>
    where
        F: Fn(&mut fmt::Formatter) -> fmt::Result,
    {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            self.0(f)
        }
    }

    DisplayWith(f)
}

#[macro_export]
macro_rules! dbg_active_nodes {
    ($geometry:expr, $active_nodes:expr) => {
        $crate::debug::debug_with($crate::debug::debug_active_nodes(
            $geometry,
            $active_nodes,
        ))
    };
}

pub fn debug_active_nodes<'a>(
    geometry: &'a Geometry,
    active_nodes: &'a ActiveNodes,
) -> impl Fn(&mut fmt::Formatter) -> fmt::Result + 'a {
    |f: &mut fmt::Formatter| -> fmt::Result {
        f.write_str(&format!(
            "[ {} ",
            active_nodes
                .items()
                .iter()
                .enumerate()
                .map(|(ix, &id)| {
                    let style = if ix == active_nodes.cursor() {
                        STYLE_CURSOR
                    } else {
                        Style::new()
                    };
                    format!(
                        "\n{}",
                        style.paint(format_args!(
                            "{:?}",
                            dbg_node!(geometry, &geometry[id])
                        ))
                    )
                },)
                .join(",")
        ))?;
        if active_nodes.len() > 0 {
            f.write_str("\n]")
        } else {
            f.write_str("]")
        }
    }
}

#[macro_export]
macro_rules! dbg_active_edges {
    (@from_dbg_decomposer $geometry:expr, $active_edges:expr, $edge_scans:expr) => {
        $crate::debug::debug_with($crate::debug::debug_active_edges(
            $geometry,
            $active_edges,
            $edge_scans,
        ))
    };
    ($geometry:expr, $active_edges:expr, $edge_scans:expr) => {
        $crate::debug::debug_with($crate::debug::debug_active_edges(
            $geometry,
            $active_edges,
            Some($edge_scans),
        ))
    };
    ($geometry:expr, $active_edges:expr) => {
        $crate::debug::debug_with($crate::debug::debug_active_edges(
            $geometry,
            $active_edges,
            None,
        ))
    };
}

pub fn debug_active_edges<'a>(
    geometry: &'a Geometry,
    active_edges: &'a ActiveEdges,
    edge_scans: Option<&'a EdgeScans>,
) -> impl Fn(&mut fmt::Formatter) -> fmt::Result + 'a {
    move |f| {
        f.write_str(&format!(
            "[ {} ",
            active_edges
                .items()
                .iter()
                .enumerate()
                .map(|(ix, &id)| {
                    let edge = geometry[id];

                    let edge_tag =
                        match edge_scans.and_then(|es| es.matches_edge(id)) {
                            Some(side) => match side {
                                EdgeTy::Left => LEFT_SCAN_EDGE.paint("le:"),
                                EdgeTy::Right => RIGHT_SCAN_EDGE.paint("re:"),
                            },
                            None => Style::new().paint(""),
                        };

                    let cursor_tag =
                        match edge_scans.and_then(|es| es.matches_cursor(ix)) {
                            Some(side) => match side {
                                EdgeTy::Left => LEFT_SCAN_EDGE.paint("lc:"),
                                EdgeTy::Right => RIGHT_SCAN_EDGE.paint("rc:"),
                            },
                            None => Style::new().paint(""),
                        };

                    format!(
                        "\n{cursor_tag}{edge_tag}{:?}",
                        dbg_edge!(geometry, &edge)
                    )
                },)
                .join(",")
        ))?;
        if active_edges.len() > 0 {
            f.write_str("\n]")
        } else {
            f.write_str("]")
        }
    }
}

#[macro_export]
macro_rules! dbg_decomposer {
    ($decomposer:expr, $geometry:expr, $edge_scans:expr) => {
        $crate::debug::debug_with($crate::debug::debug_decomposer(
            $decomposer,
            $geometry,
            $edge_scans,
        ))
    };
    ($decomposer:expr, $geometry:expr) => {
        $crate::debug::debug_with($crate::debug::debug_decomposer(
            $decomposer,
            $geometry,
            None,
        ))
    };
}

macro_rules! dbg_field {
    ($formatter:expr, $field_id:literal, $data:expr) => {
        writeln!(
            $formatter,
            "\t{}: {:?}",
            STYLE_LABEL.paint($field_id),
            $data
        )
    };
}
pub fn debug_decomposer<'a>(
    decomposer: &'a Decomposer,
    geometry: &'a Geometry,
    edge_scans: Option<&'a EdgeScans>,
) -> impl Fn(&mut fmt::Formatter) -> fmt::Result + 'a {
    move |f| {
        write!(f, "{}", type_name!("\nDecomposer {"))?;
        dbg_field!(
            f,
            "\nactive_nodes",
            dbg_active_nodes!(geometry, &decomposer.active_nodes)
        )?;
        dbg_field!(
            f,
            "\nactive_edges",
            dbg_active_edges!(@from_dbg_decomposer geometry, &decomposer.active_edges, edge_scans)
        )?;
        write!(f, "{}", type_name!("}"))
    }
}
