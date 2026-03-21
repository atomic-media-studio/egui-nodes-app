//! Playground: depends only on [`egui_nodes`]. NodeGraph lives under `egui_nodes::ui::nodes_engine`.
//! NodeGraph style UI lives in [`style_panel`] so this file stays the main entry you edit first.

mod style_panel;

use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui;
use egui_phosphor::regular;
use egui_nodes::nodes_engine::{
    canvas::{NodeGraphViewer, PinInfo},
    InPin, NodeGraph, OutPin,
};
use egui_nodes::{
    GraphChanges, InteractionMode, Layout2d, NodeData, NodesEditor, NodesShellViewer, NodesStyle,
    NodesView, NodesViewState,
};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        persist_window: true,
        ..Default::default()
    };
    eframe::run_native(
        "egui-nodes playground",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::<TemplateApp>::default())
        }),
    )
}

#[derive(Clone, PartialEq)]
enum DemoNode {
    Number(f64),
    Sink,
}

struct DemoViewer {
    editor: Rc<RefCell<NodesEditor<DemoNode, ()>>>,
}

impl DemoViewer {
    fn new(editor: Rc<RefCell<NodesEditor<DemoNode, ()>>>) -> Self {
        Self { editor }
    }
}

impl NodeGraphViewer<NodeData<DemoNode>> for DemoViewer {
    fn title(&mut self, node: &NodeData<DemoNode>) -> String {
        match &node.user {
            DemoNode::Number(_) => "Number".to_owned(),
            DemoNode::Sink => "Sink".to_owned(),
        }
    }

    fn inputs(&mut self, node: &NodeData<DemoNode>) -> usize {
        match &node.user {
            DemoNode::Number(_) => 0,
            DemoNode::Sink => 1,
        }
    }

    #[allow(refining_impl_trait)]
    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut egui::Ui,
        node_graph: &mut NodeGraph<NodeData<DemoNode>>,
    ) -> PinInfo {
        match &*pin.remotes {
            [] => ui.label("None"),
            [remote] => match &node_graph[remote.node].user {
                DemoNode::Number(value) => ui.label(format!("{value:.3}")),
                DemoNode::Sink => ui.label("Invalid"),
            },
            _ => ui.label("Multiple"),
        };
        PinInfo::circle()
    }

    fn outputs(&mut self, node: &NodeData<DemoNode>) -> usize {
        match &node.user {
            DemoNode::Number(_) => 1,
            DemoNode::Sink => 0,
        }
    }

    #[allow(refining_impl_trait)]
    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut egui::Ui,
        node_graph: &mut NodeGraph<NodeData<DemoNode>>,
    ) -> PinInfo {
        match &mut node_graph.get_node_mut(pin.id.node).unwrap().user {
            DemoNode::Number(value) => {
                ui.add(egui::DragValue::new(value).speed(0.1));
            }
            DemoNode::Sink => {
                ui.label("-");
            }
        }
        PinInfo::circle()
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _node_graph: &mut NodeGraph<NodeData<DemoNode>>) -> bool {
        true
    }

    fn show_graph_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut egui::Ui,
        _node_graph: &mut NodeGraph<NodeData<DemoNode>>,
    ) {
        if ui.button("Add Number").clicked() {
            let mut e = self.editor.borrow_mut();
            e.insert_node(
                DemoNode::Number(0.0),
                Layout2d::new(pos.x, pos.y),
                0,
                1,
            );
            ui.close();
        }
        if ui.button("Add Sink").clicked() {
            let mut e = self.editor.borrow_mut();
            e.insert_node(DemoNode::Sink, Layout2d::new(pos.x, pos.y), 1, 0);
            ui.close();
        }
    }

    fn current_transform(
        &mut self,
        _to_global: &mut egui::emath::TSTransform,
        _node_graph: &mut NodeGraph<NodeData<DemoNode>>,
    ) {
    }
}

fn init_demo(editor: &mut NodesEditor<DemoNode, ()>) {
    let a = editor.insert_node(DemoNode::Number(1.0), Layout2d::new(40.0, 40.0), 0, 1);
    let b = editor.insert_node(DemoNode::Sink, Layout2d::new(280.0, 40.0), 1, 0);
    let out_pin = editor.graph.node(a).unwrap().outputs[0].id;
    let in_pin = editor.graph.node(b).unwrap().inputs[0].id;
    editor.connect_pins(out_pin, in_pin, ()).expect("demo connect");
}

struct TemplateApp {
    editor: Rc<RefCell<NodesEditor<DemoNode, ()>>>,
    nodes_style: NodesStyle,
    view_state: NodesViewState,
    viewer: NodesShellViewer<DemoViewer>,
    /// Last drained [`GraphChanges`] summary (for shell UX; drive evaluation from the same signal).
    last_graph_changes: String,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let editor = Rc::new(RefCell::new(NodesEditor::new()));
        init_demo(&mut editor.borrow_mut());

        let mut nodes_style = NodesStyle::new();
        nodes_style.canvas = style_panel::default_canvas_style();

        let viewer = NodesShellViewer::new(DemoViewer::new(Rc::clone(&editor)));

        Self {
            editor,
            nodes_style,
            view_state: NodesViewState::default(),
            viewer,
            last_graph_changes: String::new(),
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
            ui.heading("egui-nodes playground");
            ui.horizontal(|ui| {
                let _ = ui.button(regular::ALARM);
                let _ = ui.button(regular::AIRPLANE);
            });
            ui.separator();
            ui.label("Mode");
            egui::ComboBox::from_id_salt("interaction-mode")
                .selected_text(mode_label(self.view_state.mode))
                .show_ui(ui, |ui| {
                    for m in [
                        InteractionMode::Select,
                        InteractionMode::PanZoom,
                        InteractionMode::Connect,
                        InteractionMode::InsertNode,
                        InteractionMode::EditNode,
                        InteractionMode::Inspect,
                    ] {
                        ui.selectable_value(&mut self.view_state.mode, m, mode_label(m));
                    }
                });
            if ui
                .button(if self.view_state.is_inspect() {
                    "Leave inspect (I)"
                } else {
                    "Inspect (I)"
                })
                .clicked()
                || ctx.input(|i| i.key_pressed(egui::Key::I))
            {
                self.view_state.toggle_inspect();
            }
        });

        egui::SidePanel::left("node_graph-controls")
            .resizable(true)
            .default_width(320.0)
            .min_width(220.0)
            .show(ctx, |ui| {
                ui.label("NodeGraph style (engine: egui_nodes::ui::nodes_engine)");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    style_panel::style_controls_ui(ui, &mut self.nodes_style.canvas);
                });
                ui.separator();
                ui.label("Last graph activity:");
                ui.monospace(&self.last_graph_changes);
                ui.small(
                    "Drain GraphChanges after NodesView::show to re-run Executor only when needed (topology ⇒ recompute_topo).",
                );
            });

        egui::CentralPanel::default().show(ctx, |ui| {
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

fn mode_label(m: InteractionMode) -> &'static str {
    match m {
        InteractionMode::Select => "Select",
        InteractionMode::PanZoom => "Pan / zoom",
        InteractionMode::Connect => "Connect",
        InteractionMode::InsertNode => "Insert node",
        InteractionMode::EditNode => "Edit node",
        InteractionMode::Inspect => "Inspect (read-only)",
    }
}
