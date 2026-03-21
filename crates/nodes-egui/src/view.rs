use egui::{Id, Ui};

use egui_snarl::Snarl;
use egui_snarl::ui::{SnarlViewer, SnarlWidget};

use crate::shell::NodesShellViewer;
use crate::state::NodesViewState;
use crate::style::NodesStyle;

/// Primary widget API: one call per frame inside a [`Ui`] region.
pub struct NodesView<'a> {
    pub view_state: &'a mut NodesViewState,
    pub style: &'a NodesStyle,
    snarl_widget_id: Id,
}

impl<'a> NodesView<'a> {
    pub fn new(view_state: &'a mut NodesViewState, style: &'a NodesStyle) -> Self {
        Self {
            view_state,
            style,
            snarl_widget_id: Id::new("nodes-view"),
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

    pub fn show<T, V: SnarlViewer<T>>(
        &mut self,
        snarl: &mut Snarl<T>,
        viewer: &mut NodesShellViewer<V>,
        ui: &mut Ui,
    ) -> egui::Response {
        viewer.prepare(
            self.snarl_widget_id,
            self.view_state.mode,
            ui.ctx(),
            self.style,
        );
        let snarl_style = self.style.to_snarl_style();
        SnarlWidget::new()
            .id(self.snarl_widget_id)
            .style(snarl_style)
            .show(snarl, viewer, ui)
    }
}
