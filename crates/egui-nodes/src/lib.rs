//! **egui-nodes** — egui node editor on top of [`core_graph`].
//!
//! ## Layers
//! - **`core-graph`** (re-exported) — headless [`Graph`], [`Link`], ids, optional [`Executor`](core_graph::Executor) and
//!   evaluation. No egui.
//! - **This crate** — [`NodesEditor`] / [`NodesView`], [`NodeGraph`](crate::ui::nodes_engine::NodeGraph),
//!   canvas ([`crate::ui::nodes_engine::canvas`]), and styling. [`NodeData`] maps slab nodes to
//!   [`core_graph::NodeId`].
//! - **[`ui`]** — editor session, shell viewer, and the nodes engine.

pub mod io;
pub mod layout_bridge;
pub mod ui;

pub use core_graph::{
    EvalContext, Executor, Graph, GraphError, Layout2d, Link, LinkId, Node, NodeEvaluator, NodeId,
    Pin, PinId, PinKind, Value, compute_topological_order, gather_inputs_for_node,
};
pub use io::{load_graph, save_graph};
pub use layout_bridge::{layout_to_pos2, pos2_to_layout};
pub use ui::{
    BackgroundStyle, DefaultEdgeStyleHook, DefaultNodeStyleHook, EdgeStyleHook, GraphChanges,
    GridSettings, InteractionMode, NodeData, NodeStyleHook, NodesEditor, NodesEditorError,
    NodesShellViewer, NodesStyle, NodesView, NodesViewState, PanZoomState, SelectionState,
};

/// Re-export the headless graph crate for `use egui_nodes::core_graph::…` or version pinning.
pub use core_graph;

/// Interactive node graph and canvas — alias of [`crate::ui::nodes_engine`].
pub use ui::nodes_engine;

/// Alias for [`NodesEditorError`] (older name).
pub type AdapterError = NodesEditorError;
