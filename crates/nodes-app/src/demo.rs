use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui;
use egui_nodes::egui_snarl::{InPin, OutPin, Snarl, ui::{PinInfo, SnarlViewer}};
use egui_nodes::{Layout2d, NodeData, SnarlAdapter};

#[derive(Clone)]
pub enum DemoNode {
    Number(f64),
    Sink,
}

pub struct DemoViewer {
    pub adapter: Rc<RefCell<SnarlAdapter<DemoNode, ()>>>,
    initial_zoom_pending: bool,
}

impl DemoViewer {
    pub fn new(adapter: Rc<RefCell<SnarlAdapter<DemoNode, ()>>>) -> Self {
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

pub fn init_demo(adapter: &mut SnarlAdapter<DemoNode, ()>) {
    let a = adapter.insert_node(DemoNode::Number(1.0), Layout2d::new(40.0, 40.0), 0, 1);
    let b = adapter.insert_node(DemoNode::Sink, Layout2d::new(280.0, 40.0), 1, 0);
    let out_pin = adapter.graph.node(a).unwrap().outputs[0];
    let in_pin = adapter.graph.node(b).unwrap().inputs[0];
    adapter.connect_pins(out_pin, in_pin, ()).expect("demo connect");
}
