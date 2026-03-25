//! **egui-nodes** — egui node editor on top of [`graph_lib`].
//!
//! ## Layers
//! - **`graph-lib`** (re-exported) — headless [`Graph`], [`Link`], ids, optional [`Executor`] and
//!   evaluation. No egui.
//! - **This crate** — [`NodesEditor`] / [`NodesView`], [`NodeGraph`](crate::ui::nodes_engine::NodeGraph),
//!   canvas ([`crate::ui::nodes_engine::canvas`]), and styling. [`NodeData`] maps slab nodes to
//!   [`graph_lib::NodeId`]. Preset payloads: [`DefaultNode`] and [`DefaultNodeViewer`].
//! - **[`ui`]** — editor session, shell viewer, [`NodesWorkspace`] (multi-tab graphs), and the nodes engine.

pub mod io;
pub mod ui;

pub use graph_lib::{
    EvalContext, Executor, Graph, GraphError, Layout2d, Link, LinkId, Node, NodeEvaluator, NodeId,
    Pin, PinId, PinKind, PinType, Value, compute_topological_order, dependency_graph_is_acyclic,
    gather_inputs_for_node,
};
pub use io::{load_graph, save_graph};
pub use ui::{
    BackgroundStyle, DefaultEdgeStyleHook, DefaultNode, DefaultNodeStyleHook, DefaultNodeViewer,
    EdgeStyleHook, GraphChanges, GridSettings, NodeData, NodeStyleHook, NodesEditor,
    NodesEditorError, NodesShellViewer, NodesStyle, NodesView, NodesViewState, NodesWorkspace,
    PanZoomState, SelectionState, canvas_style_controls_ui, layout_to_pos2, pin_types_for_default_node,
    pos2_to_layout, seed_default_demo_graph,
};

/// Re-export the graph crate for `use egui_nodes::graph_lib::…` or version pinning.
pub use graph_lib;

/// Interactive node graph and canvas — alias of [`crate::ui::nodes_engine`].
pub use ui::nodes_engine;

/// Alias for [`NodesEditorError`] (older name).
pub type AdapterError = NodesEditorError;
