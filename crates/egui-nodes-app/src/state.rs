/// High-level interaction mode. Snarl still handles low-level pointer routing; this value drives
/// policy (e.g. inspect/read-only) and future tools (palette insert, marquee tweaks).
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

/// Reserved for lifting pan/zoom into app state. Today Snarl stores the viewport transform internally.
#[derive(Clone, Copy, Debug, Default)]
pub struct PanZoomState;

/// Reserved for selection mirrors. Use `egui_snarl::ui::SnarlWidget::get_selected_nodes` after
/// [`NodesView::show`](crate::view::NodesView::show), then map through [`SnarlAdapter::graph_node`](crate::SnarlAdapter).
#[derive(Clone, Copy, Debug, Default)]
pub struct SelectionState;

/// View state separate from the headless [`crate::graph::Graph`]: modes and future overlays.
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
            pan_zoom: PanZoomState::default(),
            selection: SelectionState::default(),
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
