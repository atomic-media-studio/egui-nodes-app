use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui;
use nodes_snarl::egui_snarl::{InPin, OutPin, Snarl, ui::{PinInfo, SnarlViewer}};
use nodes_snarl::{Layout2d, SemanticGraph, SemanticNode, SemanticSnarlBridge};

#[derive(Clone)]
pub enum DemoNode {
    Number(f64),
    Sink,
}

pub struct DemoViewer {
    pub graph: Rc<RefCell<SemanticGraph<DemoNode, ()>>>,
    pub bridge: Rc<RefCell<SemanticSnarlBridge>>,
    initial_zoom_pending: bool,
}

impl DemoViewer {
    pub fn new(
        graph: Rc<RefCell<SemanticGraph<DemoNode, ()>>>,
        bridge: Rc<RefCell<SemanticSnarlBridge>>,
    ) -> Self {
        Self {
            graph,
            bridge,
            initial_zoom_pending: true,
        }
    }
}

impl SnarlViewer<DemoNode> for DemoViewer {
    fn title(&mut self, node: &DemoNode) -> String {
        match node {
            DemoNode::Number(_) => "Number".to_owned(),
            DemoNode::Sink => "Sink".to_owned(),
        }
    }

    fn inputs(&mut self, node: &DemoNode) -> usize {
        match node {
            DemoNode::Number(_) => 0,
            DemoNode::Sink => 1,
        }
    }

    #[allow(refining_impl_trait)]
    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<DemoNode>,
    ) -> PinInfo {
        match &*pin.remotes {
            [] => ui.label("None"),
            [remote] => match snarl[remote.node] {
                DemoNode::Number(value) => ui.label(format!("{value:.3}")),
                DemoNode::Sink => ui.label("Invalid"),
            },
            _ => ui.label("Multiple"),
        };
        PinInfo::circle()
    }

    fn outputs(&mut self, node: &DemoNode) -> usize {
        match node {
            DemoNode::Number(_) => 1,
            DemoNode::Sink => 0,
        }
    }

    #[allow(refining_impl_trait)]
    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<DemoNode>,
    ) -> PinInfo {
        match &mut snarl[pin.id.node] {
            DemoNode::Number(value) => {
                ui.add(egui::DragValue::new(value).speed(0.1));
            }
            DemoNode::Sink => {
                ui.label("-");
            }
        }
        PinInfo::circle()
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<DemoNode>) -> bool {
        true
    }

    fn show_graph_menu(&mut self, pos: egui::Pos2, ui: &mut egui::Ui, snarl: &mut Snarl<DemoNode>) {
        if ui.button("Add Number").clicked() {
            let mut g = self.graph.borrow_mut();
            let mut b = self.bridge.borrow_mut();
            b.insert_node(
                &mut *g,
                snarl,
                SemanticNode::new(DemoNode::Number(0.0), Layout2d::new(pos.x, pos.y)),
            );
            ui.close();
        }
        if ui.button("Add Sink").clicked() {
            let mut g = self.graph.borrow_mut();
            let mut b = self.bridge.borrow_mut();
            b.insert_node(
                &mut *g,
                snarl,
                SemanticNode::new(DemoNode::Sink, Layout2d::new(pos.x, pos.y)),
            );
            ui.close();
        }
    }

    fn current_transform(
        &mut self,
        to_global: &mut egui::emath::TSTransform,
        _snarl: &mut Snarl<DemoNode>,
    ) {
        if self.initial_zoom_pending && to_global.scaling > 0.0 {
            to_global.scaling *= 0.85;
            self.initial_zoom_pending = false;
        }
    }
}

pub fn init_demo_graph(
    graph: &mut SemanticGraph<DemoNode, ()>,
    bridge: &mut SemanticSnarlBridge,
    snarl: &mut Snarl<DemoNode>,
) {
    let a = bridge.insert_node(
        graph,
        snarl,
        SemanticNode::new(DemoNode::Number(1.0), Layout2d::new(40.0, 40.0)),
    );
    let b = bridge.insert_node(
        graph,
        snarl,
        SemanticNode::new(DemoNode::Sink, Layout2d::new(280.0, 40.0)),
    );
    bridge.connect(graph, snarl, a, 0, b, 0).expect("demo connect");
}
