use egui::{Id, Ui};

use egui_snarl_fork::ui::{SnarlViewer, SnarlWidget};

use crate::snarl_adapter::viewer::NodesShellViewer;
use crate::snarl_adapter::{NodeData, SnarlAdapter};
use crate::ui::state::NodesViewState;
use crate::ui::style::NodesStyle;

/// Ergonomic widget: owns the round-trip sync around [`SnarlWidget::show`](egui_snarl_fork::ui::SnarlWidget::show).
/// Implement [`SnarlViewer`](egui_snarl_fork::ui::SnarlViewer)`<`[`NodeData<N>`]`>` for your domain UI.
pub struct NodesView<'a, N, E, V> {
    pub adapter: &'a mut SnarlAdapter<N, E>,
    pub view_state: &'a mut NodesViewState,
    pub style: &'a NodesStyle,
    pub viewer: &'a mut NodesShellViewer<V>,
    snarl_widget_id: Id,
}

impl<'a, N, E, V> NodesView<'a, N, E, V> {
    pub fn new(
        adapter: &'a mut SnarlAdapter<N, E>,
        view_state: &'a mut NodesViewState,
        style: &'a NodesStyle,
        viewer: &'a mut NodesShellViewer<V>,
    ) -> Self {
        Self {
            adapter,
            view_state,
            style,
            viewer,
            snarl_widget_id: Id::new("egui-nodes-view"),
        }
    }

    pub fn with_snarl_id(mut self, id: Id) -> Self {
        self.snarl_widget_id = id;
        self
    }

    pub fn with_style(mut self, style: &'a NodesStyle) -> Self {
        self.style = style;
        self
    }

    pub fn show(&mut self, ui: &mut Ui) -> egui::Response
    where
        N: Clone + PartialEq,
        E: Default + Clone,
        V: SnarlViewer<NodeData<N>>,
    {
        self.adapter.sync_snarl_payloads_from_graph();
        self.viewer.prepare(
            self.snarl_widget_id,
            self.view_state.mode,
            ui.ctx(),
            self.style,
        );
        let snarl_style = self.style.to_snarl_style();
        let r = SnarlWidget::new()
            .id(self.snarl_widget_id)
            .style(snarl_style)
            .show(&mut self.adapter.snarl, self.viewer, ui);
        self.adapter.sync_graph_from_snarl();
        r
    }
}
