use egui::{Id, Ui};

use crate::ui::editor::shell_viewer::NodesShellViewer;
use crate::ui::editor::{NodeData, NodesEditor};
use crate::ui::nodes_canvas::{NodeGraphViewer, NodesCanvas};
use crate::ui::state::NodesViewState;
use crate::ui::style::NodesStyle;

/// Ergonomic widget: owns the round-trip sync around [`NodesCanvas::show`](crate::ui::nodes_canvas::NodesCanvas::show).
/// Implement [`NodeGraphViewer`]`<`[`NodeData<N>`]`>` for your domain UI.
pub struct NodesView<'a, N, E, V> {
    pub editor: &'a mut NodesEditor<N, E>,
    pub view_state: &'a mut NodesViewState,
    pub style: &'a NodesStyle,
    pub viewer: &'a mut NodesShellViewer<V>,
    canvas_widget_id: Id,
}

impl<'a, N, E, V> NodesView<'a, N, E, V> {
    pub fn new(
        editor: &'a mut NodesEditor<N, E>,
        view_state: &'a mut NodesViewState,
        style: &'a NodesStyle,
        viewer: &'a mut NodesShellViewer<V>,
    ) -> Self {
        Self {
            editor,
            view_state,
            style,
            viewer,
            canvas_widget_id: Id::new("egui-nodes-view"),
        }
    }

    pub fn with_canvas_id(mut self, id: Id) -> Self {
        self.canvas_widget_id = id;
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
        V: NodeGraphViewer<NodeData<N>>,
    {
        self.editor.sync_node_graph_payloads_from_graph();
        self.viewer.prepare(self.canvas_widget_id, ui.ctx(), self.style);
        let canvas_style = self.style.to_canvas_style();
        let r = NodesCanvas::new()
            .id(self.canvas_widget_id)
            .style(canvas_style)
            .show(&mut self.editor.node_graph, self.viewer, ui);
        self.editor.sync_graph_from_node_graph();
        r
    }
}
