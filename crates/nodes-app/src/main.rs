//! Lightweight template: semantic graph + Snarl bridge + `nodes-egui` shell over vendored `egui-snarl`.

mod demo;
mod style_panel;

use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui;
use egui_phosphor::regular;
use nodes_egui::{
    InteractionMode, NodesShellViewer, NodesStyle, NodesView, NodesViewState,
};
use nodes_snarl::egui_snarl::Snarl;
use nodes_snarl::{SemanticGraph, SemanticSnarlBridge};

use crate::demo::{DemoViewer, init_demo_graph};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "nodes library template",
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
    graph: Rc<RefCell<SemanticGraph<demo::DemoNode, ()>>>,
    bridge: Rc<RefCell<SemanticSnarlBridge>>,
    snarl: Snarl<demo::DemoNode>,
    nodes_style: NodesStyle,
    view_state: NodesViewState,
    viewer: NodesShellViewer<DemoViewer>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let graph = Rc::new(RefCell::new(SemanticGraph::new()));
        let bridge = Rc::new(RefCell::new(SemanticSnarlBridge::new()));
        let mut snarl = Snarl::new();
        init_demo_graph(
            &mut graph.borrow_mut(),
            &mut bridge.borrow_mut(),
            &mut snarl,
        );

        let mut nodes_style = NodesStyle::new();
        nodes_style.snarl = style_panel::default_snarl_style();

        let viewer = NodesShellViewer::new(DemoViewer::new(
            Rc::clone(&graph),
            Rc::clone(&bridge),
        ));

        Self {
            graph,
            bridge,
            snarl,
            nodes_style,
            view_state: NodesViewState::default(),
            viewer,
        }
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top-bar").show(ctx, |ui| {
            ui.heading("nodes library template");
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
                ui.label("Snarl panel (vendored egui-snarl)");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    style_panel::style_controls_ui(ui, &mut self.nodes_style.snarl);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut nodes_view = NodesView::new(&mut self.view_state, &self.nodes_style)
                .with_snarl_id(egui::Id::new("main-snarl-panel"));
            let _ = nodes_view.show(&mut self.snarl, &mut self.viewer, ui);

            let mut g = self.graph.borrow_mut();
            self.bridge
                .borrow()
                .sync_graph_from_snarl(&mut *g, &self.snarl);
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
