//! **egui-nodes** — headless [`graph::Graph`] + [`SnarlAdapter`] + [`NodesView`] so applications can
//! depend on this crate alone; the Snarl fork is an implementation detail.
//!
//! ## Layers
//! - [`graph`] — `Graph<N, E>`, [`Node`], [`Link`], pins (no egui).
//! - [`adapter`] — [`SnarlAdapter`], [`NodeData`], sync with [`egui_snarl::Snarl`].
//! - [`view`], [`state`], [`style`], [`shell`] — widget and presentation.

pub mod adapter;
pub mod graph;
pub mod io;
pub mod layout_bridge;
pub mod shell;
pub mod state;
pub mod style;
pub mod view;

pub use adapter::{AdapterError, NodeData, SnarlAdapter};
pub use graph::{Graph, GraphError, Layout2d, Link, LinkId, Node, NodeId, PinId};
pub use io::{load_graph, save_graph};
pub use layout_bridge::{layout_to_pos2, pos2_to_layout};
pub use shell::NodesShellViewer;
pub use state::{
    InteractionMode, NodesViewState, PanZoomState, SelectionState,
};
pub use style::{
    BackgroundStyle, DefaultEdgeStyleHook, DefaultNodeStyleHook, EdgeStyleHook, GridSettings,
    NodeStyleHook, NodesStyle,
};
pub use view::NodesView;

/// Re-export the Snarl engine for advanced users (custom widgets, probes). Prefer [`NodesView`] for
/// normal apps.
pub use egui_snarl;
