//! Demo app: depends only on [`egui_nodes`] — no direct Snarl imports in application code.

mod demo;
mod style_panel;

use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui;
use egui_phosphor::regular;
use egui_nodes::{
    InteractionMode, NodesShellViewer, NodesStyle, NodesView, NodesViewState, SnarlAdapter,
};

use crate::demo::{DemoViewer, init_demo};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui-nodes template",
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
    adapter: Rc<RefCell<SnarlAdapter<demo::DemoNode, ()>>>,
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
            ui.heading("egui-nodes template");
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
