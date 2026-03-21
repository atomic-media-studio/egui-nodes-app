//! Playground: depends only on [`egui_nodes`] (Snarl stays behind the adapter). Snarl style UI lives
//! in [`style_panel`] so this file stays the main entry you edit first.

mod style_panel;

use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui;
use egui_phosphor::regular;
use egui_nodes::egui_snarl_fork::{InPin, OutPin, Snarl, ui::{PinInfo, SnarlViewer}};
use egui_nodes::{
    InteractionMode, Layout2d, NodeData, NodesShellViewer, NodesStyle, NodesView, NodesViewState,
    SnarlAdapter,
};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 640.0]),
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

#[derive(Clone)]
enum DemoNode {
    Number(f64),
    Sink,
}

struct DemoViewer {
    adapter: Rc<RefCell<SnarlAdapter<DemoNode, ()>>>,
    initial_zoom_pending: bool,
}

impl DemoViewer {
    fn new(adapter: Rc<RefCell<SnarlAdapter<DemoNode, ()>>>) -> Self {
        Self {
            adapter,
            initial_zoom_pending: true,
        }
    }
}

impl SnarlViewer<NodeData<DemoNode>> for DemoViewer {
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
        snarl: &mut Snarl<NodeData<DemoNode>>,
    ) -> PinInfo {
        match &*pin.remotes {
            [] => ui.label("None"),
            [remote] => match &snarl[remote.node].user {
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
        snarl: &mut Snarl<NodeData<DemoNode>>,
    ) -> PinInfo {
        match &mut snarl.get_node_mut(pin.id.node).unwrap().user {
            DemoNode::Number(value) => {
                ui.add(egui::DragValue::new(value).speed(0.1));
            }
            DemoNode::Sink => {
                ui.label("-");
            }
        }
        PinInfo::circle()
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<NodeData<DemoNode>>) -> bool {
        true
    }

    fn show_graph_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut egui::Ui,
        _snarl: &mut Snarl<NodeData<DemoNode>>,
    ) {
        if ui.button("Add Number").clicked() {
            let mut a = self.adapter.borrow_mut();
            a.insert_node(
                DemoNode::Number(0.0),
                Layout2d::new(pos.x, pos.y),
                0,
                1,
            );
            ui.close();
        }
        if ui.button("Add Sink").clicked() {
            let mut a = self.adapter.borrow_mut();
            a.insert_node(DemoNode::Sink, Layout2d::new(pos.x, pos.y), 1, 0);
            ui.close();
        }
    }

    fn current_transform(
        &mut self,
        to_global: &mut egui::emath::TSTransform,
        _snarl: &mut Snarl<NodeData<DemoNode>>,
    ) {
        if self.initial_zoom_pending && to_global.scaling > 0.0 {
            to_global.scaling *= 0.85;
            self.initial_zoom_pending = false;
        }
    }
}

fn init_demo(adapter: &mut SnarlAdapter<DemoNode, ()>) {
    let a = adapter.insert_node(DemoNode::Number(1.0), Layout2d::new(40.0, 40.0), 0, 1);
    let b = adapter.insert_node(DemoNode::Sink, Layout2d::new(280.0, 40.0), 1, 0);
    let out_pin = adapter.graph.node(a).unwrap().outputs[0];
    let in_pin = adapter.graph.node(b).unwrap().inputs[0];
    adapter.connect_pins(out_pin, in_pin, ()).expect("demo connect");
}

struct TemplateApp {
    adapter: Rc<RefCell<SnarlAdapter<DemoNode, ()>>>,
    nodes_style: NodesStyle,
    view_state: NodesViewState,
    viewer: NodesShellViewer<DemoViewer>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let adapter = Rc::new(RefCell::new(SnarlAdapter::new()));
        init_demo(&mut adapter.borrow_mut());

        let mut nodes_style = NodesStyle::new();
        nodes_style.snarl = style_panel::default_snarl_style();

        let viewer = NodesShellViewer::new(DemoViewer::new(Rc::clone(&adapter)));

        Self {
            adapter,
            nodes_style,
            view_state: NodesViewState::default(),
            viewer,
        }
    }
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

        egui::SidePanel::left("snarl-controls")
            .resizable(true)
            .default_width(320.0)
            .min_width(220.0)
            .show(ctx, |ui| {
                ui.label("Snarl style (engine: egui-snarl-fork)");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    style_panel::style_controls_ui(ui, &mut self.nodes_style.snarl);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut adapter = self.adapter.borrow_mut();
            let mut nodes_view = NodesView::new(
                &mut *adapter,
                &mut self.view_state,
                &self.nodes_style,
                &mut self.viewer,
            )
            .with_snarl_id(egui::Id::new("main-snarl-panel"));
            let _ = nodes_view.show(ui);
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
