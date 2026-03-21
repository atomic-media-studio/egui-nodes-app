/// High-level interaction mode. The canvas handles pointer routing; this drives policy (e.g.
/// inspect/read-only) and future tools (palette insert, marquee tweaks).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum InteractionMode {
    #[default]
    Select,
    PanZoom,
    Connect,
    InsertNode,
    EditNode,
    Inspect,
}

/// Placeholder for lifting pan/zoom into app state. The viewport transform lives in per-canvas
/// [`crate::ui::nodes_engine::canvas::CanvasState`] (persisted by egui).
#[derive(Clone, Copy, Debug, Default)]
pub struct PanZoomState;

/// Reserved for selection mirrors. Use `egui_nodes::nodes_engine::canvas::NodesCanvas::get_selected_nodes` or
/// `egui_nodes::ui::nodes_canvas::NodesCanvas::get_selected_nodes` after
/// [`NodesView::show`](crate::ui::view::NodesView::show), then map through [`NodesEditor::core_node_id`](crate::NodesEditor).
#[derive(Clone, Copy, Debug, Default)]
pub struct SelectionState;

/// View state separate from the headless [`dag_lib::Graph`]: modes and future overlays.
#[derive(Clone, Debug)]
pub struct NodesViewState {
    pub mode: InteractionMode,
    pub pan_zoom: PanZoomState,
    pub selection: SelectionState,
    inspect_before: Option<InteractionMode>,
}

impl Default for NodesViewState {
    fn default() -> Self {
        Self {
            mode: InteractionMode::Select,
            pan_zoom: PanZoomState,
            selection: SelectionState,
            inspect_before: None,
        }
    }
}

impl NodesViewState {
    pub fn set_mode(&mut self, mode: InteractionMode) {
        self.mode = mode;
    }

    pub fn toggle_inspect(&mut self) {
        if self.mode == InteractionMode::Inspect {
            self.mode = self.inspect_before.unwrap_or(InteractionMode::Select);
            self.inspect_before = None;
        } else {
            self.inspect_before = Some(self.mode);
            self.mode = InteractionMode::Inspect;
        }
    }

    pub fn is_inspect(&self) -> bool {
        self.mode == InteractionMode::Inspect
    }
}
