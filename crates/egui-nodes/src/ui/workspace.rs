//! Multiple independent node graphs in one UI: a tab strip plus one [`NodesView`] per tab.
//!
//! Each tab owns a [`NodesEditor`], [`NodesViewState`], and identifiers under [`NodesWorkspace::root_id`]:
//! - **`tab_key`**: stable `u64` assigned when the tab is created (never reused in this workspace).
//! - **Canvas id**: `root_id.with(("egui-nodes-tab", tab_key))` — used for egui persistence, selection,
//!   and pan/zoom; unique in the app if `root_id` is unique.

use egui::{Context, Id, Response, Ui};

use crate::ui::editor::shell_viewer::NodesShellViewer;
use crate::ui::editor::{NodeData, NodesEditor};
use crate::ui::nodes_canvas::{NodeGraphViewer, seed_canvas_view_from};
use crate::ui::state::NodesViewState;
use crate::ui::style::NodesStyle;
use crate::ui::view::NodesView;

/// One document: headless graph + view graph + per-tab UI state.
struct NodesWorkspaceTab<N, E> {
    label: String,
    editor: NodesEditor<N, E>,
    view_state: NodesViewState,
    tab_key: u64,
}

/// Tabbed wrapper around [`NodesView`]: one full canvas per tab, shared [`NodesStyle`] and [`NodesShellViewer`].
pub struct NodesWorkspace<N, E> {
    root_id: Id,
    tabs: Vec<NodesWorkspaceTab<N, E>>,
    active: usize,
    next_tab_key: u64,
}

impl<N, E> NodesWorkspace<N, E> {
    /// Creates a workspace with one tab labeled `"Graph 1"`.
    #[must_use]
    pub fn new(root_id: Id) -> Self {
        let mut out = Self {
            root_id,
            tabs: Vec::new(),
            active: 0,
            next_tab_key: 0,
        };
        out.push_tab("Graph 1".into());
        out
    }

    #[inline]
    #[must_use]
    pub fn root_id(&self) -> Id {
        self.root_id
    }

    /// Append a new tab, make it active, and match its pan/zoom to the **first** tab (if it has
    /// stored canvas data). `label` is shown in the tab strip.
    pub fn add_tab(&mut self, ctx: &Context, label: impl Into<String>) {
        let template_key = self.tabs.first().map(|t| t.tab_key);
        let new_key = self.push_tab(label.into());
        if let Some(tk) = template_key {
            let from = self.tab_canvas_id(tk);
            let to = self.tab_canvas_id(new_key);
            seed_canvas_view_from(ctx, from, to);
        }
    }

    fn push_tab(&mut self, label: String) -> u64 {
        let tab_key = self.next_tab_key;
        self.next_tab_key = self.next_tab_key.saturating_add(1);
        self.tabs.push(NodesWorkspaceTab {
            label,
            editor: NodesEditor::new(),
            view_state: NodesViewState::default(),
            tab_key,
        });
        self.active = self.tabs.len() - 1;
        tab_key
    }

    /// Index of the active tab (0-based).
    #[inline]
    #[must_use]
    pub fn active_index(&self) -> usize {
        self.active
    }

    /// [`egui::Id`] passed to [`NodesView::with_canvas_id`] for the active tab, if any.
    #[must_use]
    pub fn active_canvas_id(&self) -> Option<Id> {
        self.tab_canvas_id_at(self.active)
    }

    /// Stable key for the active tab (see module docs).
    #[must_use]
    pub fn active_tab_key(&self) -> Option<u64> {
        self.tabs.get(self.active).map(|t| t.tab_key)
    }

    /// Canvas id for the tab at `index` (same as [`NodesView::with_canvas_id`] for that tab).
    #[must_use]
    pub fn tab_canvas_id_at(&self, index: usize) -> Option<Id> {
        let t = self.tabs.get(index)?;
        Some(self.tab_canvas_id(t.tab_key))
    }

    /// Stable `tab_key` for the tab at `index`.
    #[must_use]
    pub fn tab_key_at(&self, index: usize) -> Option<u64> {
        self.tabs.get(index).map(|t| t.tab_key)
    }

    #[inline]
    fn tab_canvas_id(&self, tab_key: u64) -> Id {
        self.root_id.with(("egui-nodes-tab", tab_key))
    }

    /// Read the active tab’s [`NodesEditor`].
    #[must_use]
    pub fn active_editor(&self) -> Option<&NodesEditor<N, E>> {
        self.tabs.get(self.active).map(|t| &t.editor)
    }

    /// Mutable access to the active tab’s [`NodesEditor`].
    #[must_use]
    pub fn active_editor_mut(&mut self) -> Option<&mut NodesEditor<N, E>> {
        self.tabs.get_mut(self.active).map(|t| &mut t.editor)
    }

    /// Renders the tab bar and the active tab’s [`NodesView`].
    pub fn show<V>(&mut self, ui: &mut Ui, style: &NodesStyle, viewer: &mut NodesShellViewer<V>) -> Response
    where
        N: Clone + PartialEq,
        E: Default + Clone,
        V: NodeGraphViewer<NodeData<N>>,
    {
        // Match selectable tabs to neutral widget grays instead of the theme’s selection accent.
        // Selected tabs use `selection.stroke` as label color (`Style::interact_selectable`).
        ui.scope(|ui| {
            let visuals = ui.visuals_mut();
            let inactive = visuals.widgets.inactive;
            visuals.selection.bg_fill = inactive.weak_bg_fill;
            visuals.selection.stroke = inactive.fg_stroke;

            ui.horizontal(|ui| {
                let len = self.tabs.len();
                for i in 0..len {
                    let is_active = self.active == i;
                    let label = self.tabs[i].label.as_str();
                    if ui.selectable_label(is_active, label).clicked() {
                        self.active = i;
                    }
                }
                if ui.button("+").clicked() {
                    let n = self.tabs.len() + 1;
                    self.add_tab(ui.ctx(), format!("Graph {n}"));
                }
            });
        });

        ui.separator();

        let Some(tab_key) = self.tabs.get(self.active).map(|t| t.tab_key) else {
            return ui.label("No graph tab");
        };
        let canvas_id = self.tab_canvas_id(tab_key);

        let Some(tab) = self.tabs.get_mut(self.active) else {
            return ui.label("No graph tab");
        };

        NodesView::new(
            &mut tab.editor,
            &mut tab.view_state,
            style,
            viewer,
        )
        .with_canvas_id(canvas_id)
        .show(ui)
    }
}
