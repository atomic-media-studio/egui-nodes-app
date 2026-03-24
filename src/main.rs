//! Playground: depends only on [`egui_nodes`]. Uses library [`DefaultNode`] / [`DefaultNodeViewer`].
//! Canvas style tuning uses [`egui_nodes::canvas_style_controls_ui`] from the library.

use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui;
use egui_nodes::{
    DefaultNode, DefaultNodeViewer, GraphChanges, NodesEditor, NodesShellViewer, NodesStyle,
    NodesView, NodesViewState, canvas_style_controls_ui, seed_default_demo_graph,
};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 820.0]),
        persist_window: true,
        ..Default::default()
    };
    eframe::run_native(
        "graph-lib and egui-nodes playground",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::<TemplateApp>::default())
        }),
    )
}

struct TemplateApp {
    editor: Rc<RefCell<NodesEditor<DefaultNode, ()>>>,
    nodes_style: NodesStyle,
    view_state: NodesViewState,
    viewer: NodesShellViewer<DefaultNodeViewer>,
    /// Last drained [`GraphChanges`] summary (for shell UX; drive evaluation from the same signal).
    last_graph_changes: String,
    /// Last node type spawned from the graph context menu (also printed to stdout).
    last_menu_spawn: Rc<RefCell<String>>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let editor = Rc::new(RefCell::new(NodesEditor::new()));
        seed_default_demo_graph(&mut editor.borrow_mut());

        let nodes_style = NodesStyle::with_editor_canvas_defaults();

        let last_menu_spawn = Rc::new(RefCell::new(String::new()));
        let viewer = NodesShellViewer::new(DefaultNodeViewer::new(
            Rc::clone(&editor),
            Rc::clone(&last_menu_spawn),
        ));

        Self {
            editor,
            nodes_style,
            view_state: NodesViewState::default(),
            viewer,
            last_graph_changes: String::new(),
            last_menu_spawn,
        }
    }
}

fn format_graph_changes(c: &GraphChanges) -> String {
    if !c.any() {
        return "none (idle)".to_string();
    }
    let mut parts = Vec::new();
    if c.topology_changed {
        parts.push("topology");
    }
    if c.payload_or_layout_changed {
        parts.push("payload/layout");
    }
    parts.join(" + ")
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top-bar").show(ctx, |ui| {
            ui.heading("graph-lib and egui-nodes playground");
            ui.add_space(10.0);
        });

        egui::SidePanel::left("node_graph-controls")
            .resizable(true)
            .default_width(320.0)
            .min_width(220.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    canvas_style_controls_ui(ui, &mut self.nodes_style.canvas);
                });
                ui.separator();
                ui.label("Last graph activity:");
                ui.monospace(&self.last_graph_changes);

                ui.separator();
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Last spawn from graph menu:");
                let s = self.last_menu_spawn.borrow();
                if s.is_empty() {
                    ui.weak("(none yet)");
                } else {
                    ui.monospace(s.as_str());
                }
            });
            ui.separator();

            let mut ed = self.editor.borrow_mut();
            let mut nodes_view = NodesView::new(
                &mut *ed,
                &mut self.view_state,
                &self.nodes_style,
                &mut self.viewer,
            )
            .with_canvas_id(egui::Id::new("main-nodes-canvas-panel"));
            let _ = nodes_view.show(ui);
            let changes = ed.take_graph_changes();
            self.last_graph_changes = format_graph_changes(&changes);
        });
    }
}
