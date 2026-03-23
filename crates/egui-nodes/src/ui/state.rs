//! UI-only state carried alongside [`crate::ui::view::NodesView`] (separate from graph data).

/// Placeholder for lifting pan/zoom into app state. The viewport transform lives in per-canvas
/// [`crate::ui::nodes_engine::canvas::CanvasState`] (persisted by egui).
#[derive(Clone, Copy, Debug, Default)]
pub struct PanZoomState;

/// Reserved for selection mirrors. Use `egui_nodes::nodes_engine::canvas::NodesCanvas::get_selected_nodes` or
/// `egui_nodes::ui::nodes_canvas::NodesCanvas::get_selected_nodes` after
/// [`crate::ui::view::NodesView::show`](crate::ui::view::NodesView::show), then map through [`crate::ui::editor::NodesEditor::core_node_id`](crate::ui::editor::NodesEditor).
#[derive(Clone, Copy, Debug, Default)]
pub struct SelectionState;

/// View state separate from the headless [`dag_lib::Graph`]: reserved for future overlays.
#[derive(Clone, Debug, Default)]
pub struct NodesViewState {
    pub pan_zoom: PanZoomState,
    pub selection: SelectionState,
}
