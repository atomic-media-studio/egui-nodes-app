//! Playground: depends only on [`egui_nodes`]. Uses library [`DefaultNode`] / [`DefaultNodeViewer`].
//! Canvas style tuning uses [`egui_nodes::canvas_style_controls_ui`] from the library.

use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui;
use egui_nodes::{
    DefaultNode, DefaultNodeViewer, GraphChanges, NodeData, NodesEditor, NodesShellViewer,
    NodesStyle, NodesView, NodesViewState, canvas_style_controls_ui, seed_default_demo_graph,
};
use egui_nodes::nodes_engine::canvas::get_selected_nodes;
use egui_nodes::nodes_engine::{NodeGraph, NodeId};

/// Must match [`NodesView::with_canvas_id`] for the main graph.
#[inline]
fn main_nodes_canvas_id() -> egui::Id {
    egui::Id::new("main-nodes-canvas-panel")
}

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

fn default_node_kind_label(user: &DefaultNode) -> &'static str {
    match user {
        DefaultNode::Button => "Button",
        DefaultNode::Int(_) => "Int",
        DefaultNode::Str(_) => "String",
        DefaultNode::Float(_) => "Float",
        DefaultNode::Sink => "Sink",
    }
}

fn default_node_payload_line(user: &DefaultNode) -> String {
    match user {
        DefaultNode::Button => "Payload: —".to_owned(),
        DefaultNode::Int(v) => format!("Payload: Int = {v}"),
        DefaultNode::Str(s) if s.len() <= 32 => {
            format!("Payload: String = \"{s}\"")
        }
        DefaultNode::Str(s) => format!("Payload: String ({} chars)", s.len()),
        DefaultNode::Float(v) => format!("Payload: Float = {v:.6}"),
        DefaultNode::Sink => "Payload: Sink (input only)".to_owned(),
    }
}

/// Three lines for the Node Inspector (selection comes from canvas memory; may trail by one frame).
fn node_inspector_lines(
    selected: &[NodeId],
    node_graph: &NodeGraph<NodeData<DefaultNode>>,
) -> [String; 3] {
    match selected.len() {
        0 => [
            "No node selected.".to_owned(),
            "Click a node on the canvas.".to_owned(),
            "—".to_owned(),
        ],
        n if n > 1 => [
            format!("{n} nodes selected."),
            "Inspector shows one node; select a single node.".to_owned(),
            "—".to_owned(),
        ],
        _ => {
            let id = selected[0];
            let Some(info) = node_graph.get_node_info(id) else {
                return [
                    "Selection is stale.".to_owned(),
                    "Try clicking the canvas again.".to_owned(),
                    "—".to_owned(),
                ];
            };
            let kind = default_node_kind_label(&info.value.user);
            [
                format!("Node #{} — {kind}", id.0),
                format!(
                    "Position (graph space): ({:.1}, {:.1})",
                    info.pos.x, info.pos.y
                ),
                default_node_payload_line(&info.value.user),
            ]
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
                ui.checkbox(
                    &mut self.nodes_style.canvas.snap_nodes_to_grid,
                    "Snap nodes to grid when dragging",
                );
                ui.separator();
                ui.heading("Node Inspector");
                {
                    let selected = get_selected_nodes(main_nodes_canvas_id(), ui.ctx());
                    let ed = self.editor.borrow();
                    let lines = node_inspector_lines(&selected, &ed.node_graph);
                    ui.label(&lines[0]);
                    ui.label(&lines[1]);
                    ui.label(&lines[2]);
                }
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
            .with_canvas_id(main_nodes_canvas_id());
            let _ = nodes_view.show(ui);
            let changes = ed.take_graph_changes();
            self.last_graph_changes = format_graph_changes(&changes);
        });
    }
}
