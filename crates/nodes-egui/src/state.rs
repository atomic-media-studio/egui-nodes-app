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

/// View state kept separate from the semantic graph: modes, later marquee/overlay flags, etc.
/// Selection and pan/zoom live in egui-snarl’s persisted widget state; use
/// [`egui_snarl::ui::SnarlWidget::get_selected_nodes`] after `NodesView::show` when you need IDs.
#[derive(Clone, Debug)]
pub struct NodesViewState {
    pub mode: InteractionMode,
    inspect_before: Option<InteractionMode>,
}

impl Default for NodesViewState {
    fn default() -> Self {
        Self {
            mode: InteractionMode::Select,
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
