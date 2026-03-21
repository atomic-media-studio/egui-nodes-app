//! Product-style node editor API on top of [`egui_snarl`]: view state, modes, styling hooks, and a
//! thin shell around [`egui_snarl::ui::SnarlViewer`] for selection-aware strokes and read-only inspect.

pub mod shell;
pub mod state;
pub mod style;
pub mod view;

pub use shell::NodesShellViewer;
pub use state::{InteractionMode, NodesViewState};
pub use style::{
    BackgroundStyle, DefaultEdgeStyleHook, DefaultNodeStyleHook, EdgeStyleHook, GridSettings,
    NodeStyleHook, NodesStyle,
};
pub use view::NodesView;
