//! **egui-nodes** — [`NodesEditor`] + [`NodesView`] on top of [`core_graph`] and [`ui::nodes_engine`].
//! The portable graph model lives in **`core-graph`**; this crate is the egui / nodes layer.
//!
//! ## Layers
//! - **`core-graph`** — `Graph<N, E>`, [`Node`], [`Link`], ids, [`Executor`](core_graph::Executor) (dependency; re-exported below).
//! - [`NodesEditor`](crate::NodesEditor), [`NodeData`], sync with [`Snarl`](crate::ui::nodes_engine::Snarl).
//! - [`ui`] — graph engine, editor session, canvas, view state, and styling.

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

/// Same module layout as the old `egui-snarl-fork` crate: graph types plus a `ui` submodule for the
/// widget (`SnarlWidget`, `SnarlStyle`, …).
pub mod egui_snarl_fork {
    pub use crate::ui::nodes_engine::*;
    pub mod ui {
        pub use crate::ui::nodes_engine::canvas::*;
    }
}

/// Previous name for [`NodesEditor`].
pub type SnarlAdapter<N, E> = NodesEditor<N, E>;

/// Previous name for [`NodesEditorError`].
pub type AdapterError = NodesEditorError;
