//! Editor UI — `editor` (core graph ↔ `NodeGraph`), `nodes_engine` (canvas), `view`, `style`, `state`.

pub mod editor;
pub mod nodes_engine;
pub use nodes_engine::canvas as nodes_canvas;
pub mod state;
pub mod style;
pub mod view;

pub use editor::shell_viewer::NodesShellViewer;
pub use editor::{
    GraphChanges, NodeData, NodesEditor, NodesEditorError, layout_to_pos2, pos2_to_layout,
};
pub use state::{
    NodesViewState, PanZoomState, SelectionState,
};
pub use style::{
    BackgroundStyle, DefaultEdgeStyleHook, DefaultNodeStyleHook, EdgeStyleHook, GridSettings,
    NodeStyleHook, NodesStyle,
};
pub use view::NodesView;
