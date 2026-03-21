//! **egui-nodes** — [`SnarlAdapter`] + [`NodesView`] on top of [`core_graph`] and [`egui_snarl_fork`].
//! The portable graph model lives in **`core-graph`**; this crate is the egui / Snarl layer.
//!
//! ## Layers
//! - **`core-graph`** — `Graph<N, E>`, [`Node`], [`Link`], ids, [`Executor`](core_graph::Executor) (dependency; re-exported below).
//! - [`snarl_adapter`] — [`SnarlAdapter`], [`NodeData`], sync with [`egui_snarl_fork::Snarl`].
//! - [`ui`] — widget, state, and presentation hooks.

pub mod io;
pub mod layout_bridge;
pub mod snarl_adapter;
pub mod ui;

pub use core_graph::{
    EvalContext, Executor, Graph, GraphError, Layout2d, Link, LinkId, Node, NodeEvaluator, NodeId,
    Pin, PinId, PinKind, Value, compute_topological_order, gather_inputs_for_node,
};
pub use io::{load_graph, save_graph};
pub use layout_bridge::{layout_to_pos2, pos2_to_layout};
pub use snarl_adapter::viewer::NodesShellViewer;
pub use snarl_adapter::{AdapterError, GraphChanges, NodeData, SnarlAdapter};
pub use ui::{
    BackgroundStyle, DefaultEdgeStyleHook, DefaultNodeStyleHook, EdgeStyleHook, GridSettings,
    InteractionMode, NodeStyleHook, NodesStyle, NodesView, NodesViewState, PanZoomState,
    SelectionState,
};

/// Re-export the headless graph crate for `use egui_nodes::core_graph::…` or version pinning.
pub use core_graph;

/// Re-export the Snarl engine for advanced users (custom widgets, probes). Prefer [`NodesView`] for
/// normal apps.
pub use egui_snarl_fork;
