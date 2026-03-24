//! Built-in **default** node payloads and a [`NodeGraphViewer`] that renders them.
//!
//! Extend [`DefaultNode`] with new variants here as you add more preset node types; the playground
//! and embedding apps can use [`DefaultNodeViewer`] with a [`NodesEditor`] without reimplementing
//! chrome, graph menu, or pin UI for this set.

use std::cell::RefCell;
use std::rc::Rc;

use egui::{Pos2, Ui, emath::TSTransform};

use super::nodes_engine::{
    InPin, NodeGraph, OutPin,
    canvas::{
        NodeGraphViewer, PinInfo, apply_graph_menu_width, print_graph_menu_button_clicked,
        print_graph_menu_float_clicked, print_graph_menu_int_clicked, print_graph_menu_sink_clicked,
        print_graph_menu_string_clicked,
    },
};
use crate::{Layout2d, NodeData, NodesEditor};

/// Preset node kinds shipped with **egui-nodes** (Button, Int, String, Float, Sink).
///
/// Add new variants in this crate when you expand the default palette; applications embed
/// [`NodeData`]`<`[`DefaultNode`]`>` in the canvas.
#[derive(Clone, PartialEq)]
pub enum DefaultNode {
    Button,
    Int(i32),
    Str(String),
    Float(f64),
    Sink,
}

/// [`NodeGraphViewer`] for [`DefaultNode`]: titles, pins, body widgets, and the empty-canvas graph menu.
///
/// Holds a shared [`NodesEditor`] so the graph menu can [`NodesEditor::insert_node`], and an optional
/// `last_menu_spawn` string for UI that shows which type was spawned last.
pub struct DefaultNodeViewer {
    editor: Rc<RefCell<NodesEditor<DefaultNode, ()>>>,
    last_menu_spawn: Rc<RefCell<String>>,
}

impl DefaultNodeViewer {
    /// `last_menu_spawn` is cleared and updated whenever a graph-menu entry spawns a node.
    #[must_use]
    pub fn new(
        editor: Rc<RefCell<NodesEditor<DefaultNode, ()>>>,
        last_menu_spawn: Rc<RefCell<String>>,
    ) -> Self {
        Self {
            editor,
            last_menu_spawn,
        }
    }

    fn remember_last_menu_spawn(&self, name: &'static str) {
        self.last_menu_spawn.borrow_mut().clear();
        self.last_menu_spawn.borrow_mut().push_str(name);
    }
}

impl NodeGraphViewer<NodeData<DefaultNode>> for DefaultNodeViewer {
    fn title(&mut self, node: &NodeData<DefaultNode>) -> String {
        match &node.user {
            DefaultNode::Button => "Button".to_owned(),
            DefaultNode::Int(_) => "Int".to_owned(),
            DefaultNode::Str(_) => "String".to_owned(),
            DefaultNode::Float(_) => "Float".to_owned(),
            DefaultNode::Sink => "Sink".to_owned(),
        }
    }

    fn inputs(&mut self, node: &NodeData<DefaultNode>) -> usize {
        match &node.user {
            DefaultNode::Sink => 1,
            _ => 0,
        }
    }

    #[allow(refining_impl_trait)]
    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        node_graph: &mut NodeGraph<NodeData<DefaultNode>>,
    ) -> PinInfo {
        match &*pin.remotes {
            [] => ui.label("None"),
            [remote] => match &node_graph[remote.node].user {
                DefaultNode::Float(value) => ui.label(format!("{value:.3}")),
                DefaultNode::Int(v) => ui.label(format!("{v}")),
                DefaultNode::Str(s) => ui.label(s.as_str()),
                _ => ui.label("—"),
            },
            _ => ui.label("Multiple"),
        };
        PinInfo::circle()
    }

    fn outputs(&mut self, node: &NodeData<DefaultNode>) -> usize {
        match &node.user {
            DefaultNode::Sink => 0,
            _ => 1,
        }
    }

    #[allow(refining_impl_trait)]
    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut Ui,
        node_graph: &mut NodeGraph<NodeData<DefaultNode>>,
    ) -> PinInfo {
        match &mut node_graph.get_node_mut(pin.id.node).unwrap().user {
            DefaultNode::Button => {
                let _ = ui.button("Click");
            }
            DefaultNode::Int(value) => {
                ui.add(egui::DragValue::new(value));
            }
            DefaultNode::Str(value) => {
                ui.text_edit_singleline(value);
            }
            DefaultNode::Float(value) => {
                ui.add(egui::DragValue::new(value).speed(0.1));
            }
            DefaultNode::Sink => {
                ui.label("-");
            }
        }
        PinInfo::circle()
    }

    fn has_graph_menu(
        &mut self,
        _pos: Pos2,
        _node_graph: &mut NodeGraph<NodeData<DefaultNode>>,
    ) -> bool {
        true
    }

    fn show_graph_menu(
        &mut self,
        pos: Pos2,
        ui: &mut Ui,
        _node_graph: &mut NodeGraph<NodeData<DefaultNode>>,
    ) {
        apply_graph_menu_width(ui);

        if ui.button("Button").clicked() {
            print_graph_menu_button_clicked();
            self.remember_last_menu_spawn("Button");
            let mut e = self.editor.borrow_mut();
            e.insert_node(DefaultNode::Button, Layout2d::new(pos.x, pos.y), 0, 1);
            ui.close();
        }
        if ui.button("Int").clicked() {
            print_graph_menu_int_clicked();
            self.remember_last_menu_spawn("Int");
            let mut e = self.editor.borrow_mut();
            e.insert_node(DefaultNode::Int(0), Layout2d::new(pos.x, pos.y), 0, 1);
            ui.close();
        }
        if ui.button("String").clicked() {
            print_graph_menu_string_clicked();
            self.remember_last_menu_spawn("String");
            let mut e = self.editor.borrow_mut();
            e.insert_node(DefaultNode::Str(String::new()), Layout2d::new(pos.x, pos.y), 0, 1);
            ui.close();
        }
        if ui.button("Float").clicked() {
            print_graph_menu_float_clicked();
            self.remember_last_menu_spawn("Float");
            let mut e = self.editor.borrow_mut();
            e.insert_node(DefaultNode::Float(0.0), Layout2d::new(pos.x, pos.y), 0, 1);
            ui.close();
        }
        if ui.button("Sink").clicked() {
            print_graph_menu_sink_clicked();
            self.remember_last_menu_spawn("Sink");
            let mut e = self.editor.borrow_mut();
            e.insert_node(DefaultNode::Sink, Layout2d::new(pos.x, pos.y), 1, 0);
            ui.close();
        }
    }

    fn current_transform(
        &mut self,
        _to_global: &mut TSTransform,
        _node_graph: &mut NodeGraph<NodeData<DefaultNode>>,
    ) {
    }
}

/// Small sample graph: one Float node connected to a Sink (for demos and tests).
pub fn seed_default_demo_graph(editor: &mut NodesEditor<DefaultNode, ()>) {
    let a = editor.insert_node(DefaultNode::Float(1.0), Layout2d::new(40.0, 40.0), 0, 1);
    let b = editor.insert_node(DefaultNode::Sink, Layout2d::new(280.0, 40.0), 1, 0);
    let out_pin = editor.graph.node(a).unwrap().outputs[0].id;
    let in_pin = editor.graph.node(b).unwrap().inputs[0].id;
    editor
        .connect_pins(out_pin, in_pin, ())
        .expect("default demo connect");
}
