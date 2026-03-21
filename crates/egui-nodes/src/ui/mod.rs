//! Editor UI — `editor` (core graph ↔ `NodeGraph`), `nodes_engine` (canvas), `view`, `style`, `state`.

pub mod editor;
pub mod nodes_engine;
pub use nodes_engine::canvas as nodes_canvas;
pub mod state;
pub mod style;
pub mod view;

pub use editor::shell_viewer::NodesShellViewer;
pub use editor::{
    GraphChanges, NodeData, NodesEditor, NodesEditorError,
};
pub use state::{
    InteractionMode, NodesViewState, PanZoomState, SelectionState,
};
pub use style::{
    BackgroundStyle, DefaultEdgeStyleHook, DefaultNodeStyleHook, EdgeStyleHook, GridSettings,
    NodeStyleHook, NodesStyle,
};
pub use view::NodesView;
