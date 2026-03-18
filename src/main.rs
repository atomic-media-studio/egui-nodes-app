use eframe::egui;
use egui_phosphor::regular;
use egui_snarl::{
    InPin, InPinId, OutPin, OutPinId, Snarl,
    ui::{
        BackgroundPattern, Grid, NodeLayout, PinInfo, PinPlacement, PinShape,
        SelectionStyle, SnarlStyle, SnarlViewer, SnarlWidget, WireLayer, WireStyle,
    },
};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui nodes cross-platform template",
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
    initial_zoom_pending: bool,
}

impl Default for DemoViewer {
    fn default() -> Self {
        Self {
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
            snarl.insert_node(pos, DemoNode::Number(0.0));
            ui.close();
        }
        if ui.button("Add Sink").clicked() {
            snarl.insert_node(pos, DemoNode::Sink);
            ui.close();
        }
    }

    fn current_transform(
        &mut self,
        to_global: &mut egui::emath::TSTransform,
        _snarl: &mut Snarl<DemoNode>,
    ) {
        // Start slightly zoomed out, only once per app launch.
        if self.initial_zoom_pending && to_global.scaling > 0.0 {
            to_global.scaling *= 0.85;
            self.initial_zoom_pending = false;
        }
    }
}

struct TemplateApp {
    snarl: Snarl<DemoNode>,
    style: SnarlStyle,
    viewer: DemoViewer,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let mut snarl = Snarl::new();
        let number = snarl.insert_node(egui::pos2(40.0, 40.0), DemoNode::Number(1.0));
        let sink = snarl.insert_node(egui::pos2(280.0, 40.0), DemoNode::Sink);
        let _ = snarl.connect(
            OutPinId {
                node: number,
                output: 0,
            },
            InPinId {
                node: sink,
                input: 0,
            },
        );
        Self {
            snarl,
            style: default_snarl_style(),
            viewer: DemoViewer::default(),
        }
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top-bar").show(ctx, |ui| {
            ui.heading("egui nodes application template");
            ui.label(format!(
                "Phosphor Icons: {} {}",
                regular::ALARM,
                regular::AIRPLANE
            ));
            ui.horizontal(|ui| {
                let _ = ui.button(regular::ALARM);
                let _ = ui.button(regular::AIRPLANE);
            });
        });

        egui::SidePanel::left("snarl-controls")
            .resizable(true)
            .default_width(320.0)
            .min_width(220.0)
            .show(ctx, |ui| {
                ui.label("Node panel controls");
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    style_controls_ui(ui, &mut self.style);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            SnarlWidget::new()
                .id(egui::Id::new("main-snarl-panel"))
                .style(self.style)
                .show(&mut self.snarl, &mut self.viewer, ui);
        });
    }
}

fn default_snarl_style() -> SnarlStyle {
    SnarlStyle {
        node_layout: Some(NodeLayout::coil()),
        collapsible: Some(true),
        pin_size: Some(8.0),
        pin_shape: Some(PinShape::Circle),
        pin_placement: Some(PinPlacement::Edge),
        wire_width: Some(2.0),
        wire_frame_size: Some(32.0),
        downscale_wire_frame: Some(true),
        upscale_wire_frame: Some(false),
        wire_style: Some(WireStyle::Bezier5),
        wire_layer: Some(WireLayer::BehindNodes),
        bg_pattern: Some(BackgroundPattern::grid(egui::vec2(50.0, 50.0), 0.0)),
        min_scale: Some(0.1),
        max_scale: Some(2.0),
        centering: Some(true),
        wire_smoothness: Some(1.0),
        ..SnarlStyle::new()
    }
}

fn edit_margin(ui: &mut egui::Ui, label: &str, margin: &mut egui::Margin) {
    ui.label(label);
    ui.horizontal(|ui| {
        ui.label("L");
        ui.add(egui::DragValue::new(&mut margin.left).speed(0.25).range(-64..=64));
        ui.label("R");
        ui.add(
            egui::DragValue::new(&mut margin.right)
                .speed(0.25)
                .range(-64..=64),
        );
        ui.label("T");
        ui.add(egui::DragValue::new(&mut margin.top).speed(0.25).range(-64..=64));
        ui.label("B");
        ui.add(
            egui::DragValue::new(&mut margin.bottom)
                .speed(0.25)
                .range(-64..=64),
        );
    });
}

fn edit_corner_radius(ui: &mut egui::Ui, label: &str, radius: &mut egui::CornerRadius) {
    ui.label(label);
    ui.horizontal(|ui| {
        ui.label("NW");
        ui.add(egui::DragValue::new(&mut radius.nw).range(0..=64));
        ui.label("NE");
        ui.add(egui::DragValue::new(&mut radius.ne).range(0..=64));
        ui.label("SW");
        ui.add(egui::DragValue::new(&mut radius.sw).range(0..=64));
        ui.label("SE");
        ui.add(egui::DragValue::new(&mut radius.se).range(0..=64));
    });
}

fn style_controls_ui(ui: &mut egui::Ui, style: &mut SnarlStyle) {
    ui.heading("Graph style");
    if ui.button("Reset to defaults").clicked() {
        *style = default_snarl_style();
    }
    ui.separator();

    ui.collapsing("Node layout", |ui| {
        let mut layout = style.node_layout.unwrap_or(NodeLayout::coil());
        ui.add(egui::Slider::new(&mut layout.min_pin_row_height, 0.0..=60.0).text("Min pin row"));
        style.node_layout = Some(layout);
        ui.checkbox(style.collapsible.get_or_insert(true), "Collapsible nodes");
        let header_drag_space = style.header_drag_space.get_or_insert(egui::vec2(16.0, 16.0));
        ui.add(egui::Slider::new(&mut header_drag_space.x, 0.0..=120.0).text("Header drag X"));
        ui.add(egui::Slider::new(&mut header_drag_space.y, 0.0..=120.0).text("Header drag Y"));
    });

    ui.separator();
    ui.collapsing("Pins", |ui| {
        ui.add(egui::Slider::new(style.pin_size.get_or_insert(8.0), 2.0..=24.0).text("Pin size"));

        let mut outside_margin = 8.0;
        let mut placement_kind = match style.pin_placement.unwrap_or(PinPlacement::Edge) {
            PinPlacement::Inside => 0,
            PinPlacement::Edge => 1,
            PinPlacement::Outside { margin } => {
                outside_margin = margin;
                2
            }
        };
        egui::ComboBox::from_label("Pin placement")
            .selected_text(match placement_kind {
                0 => "Inside",
                1 => "Edge",
                _ => "Outside",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut placement_kind, 0, "Inside");
                ui.selectable_value(&mut placement_kind, 1, "Edge");
                ui.selectable_value(&mut placement_kind, 2, "Outside");
            });
        if placement_kind == 2 {
            ui.add(egui::Slider::new(&mut outside_margin, 0.0..=40.0).text("Outside margin"));
        }
        style.pin_placement = Some(match placement_kind {
            0 => PinPlacement::Inside,
            1 => PinPlacement::Edge,
            _ => PinPlacement::Outside {
                margin: outside_margin,
            },
        });

        ui.horizontal(|ui| {
            ui.label("Pin fill");
            let color = style.pin_fill.get_or_insert(egui::Color32::from_rgb(120, 140, 255));
            ui.color_edit_button_srgba(color);
        });

        let stroke = style
            .pin_stroke
            .get_or_insert(egui::Stroke::new(1.5, egui::Color32::WHITE));
        ui.add(egui::Slider::new(&mut stroke.width, 0.0..=8.0).text("Pin stroke width"));
        ui.horizontal(|ui| {
            ui.label("Pin stroke color");
            ui.color_edit_button_srgba(&mut stroke.color);
        });
    });

    ui.separator();
    ui.collapsing("Wires", |ui| {
        ui.add(egui::Slider::new(style.wire_width.get_or_insert(2.0), 0.2..=10.0).text("Width"));
        ui.add(
            egui::Slider::new(style.wire_frame_size.get_or_insert(32.0), 4.0..=120.0)
                .text("Frame size"),
        );
        ui.add(
            egui::Slider::new(style.wire_smoothness.get_or_insert(1.0), 0.0..=10.0)
                .text("Smoothness"),
        );
        ui.checkbox(
            style.downscale_wire_frame.get_or_insert(true),
            "Downscale frame when close",
        );
        ui.checkbox(
            style.upscale_wire_frame.get_or_insert(false),
            "Upscale frame when far",
        );

        let mut corner_radius = match style.wire_style.unwrap_or(WireStyle::Bezier5) {
            WireStyle::AxisAligned { corner_radius } => corner_radius,
            _ => 12.0,
        };
        let mut wire_style_kind = match style.wire_style.unwrap_or(WireStyle::Bezier5) {
            WireStyle::Line => 0,
            WireStyle::AxisAligned { .. } => 1,
            WireStyle::Bezier3 => 2,
            WireStyle::Bezier5 => 3,
        };
        egui::ComboBox::from_label("Wire style")
            .selected_text(match wire_style_kind {
                0 => "Line",
                1 => "AxisAligned",
                2 => "Bezier3",
                _ => "Bezier5",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut wire_style_kind, 0, "Line");
                ui.selectable_value(&mut wire_style_kind, 1, "AxisAligned");
                ui.selectable_value(&mut wire_style_kind, 2, "Bezier3");
                ui.selectable_value(&mut wire_style_kind, 3, "Bezier5");
            });
        if wire_style_kind == 1 {
            ui.add(egui::Slider::new(&mut corner_radius, 0.0..=40.0).text("Corner radius"));
        }
        style.wire_style = Some(match wire_style_kind {
            0 => WireStyle::Line,
            1 => WireStyle::AxisAligned { corner_radius },
            2 => WireStyle::Bezier3,
            _ => WireStyle::Bezier5,
        });

        let mut wire_layer = style.wire_layer.unwrap_or(WireLayer::BehindNodes);
        egui::ComboBox::from_label("Wire layer")
            .selected_text(match wire_layer {
                WireLayer::BehindNodes => "Behind nodes",
                WireLayer::AboveNodes => "Above nodes",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut wire_layer, WireLayer::BehindNodes, "Behind nodes");
                ui.selectable_value(&mut wire_layer, WireLayer::AboveNodes, "Above nodes");
            });
        style.wire_layer = Some(wire_layer);
    });

    ui.separator();
    ui.collapsing("Background and colors", |ui| {
        let mut pattern = style.bg_pattern.unwrap_or(BackgroundPattern::new());
        let mut pattern_kind = match pattern {
            BackgroundPattern::NoPattern => 0,
            BackgroundPattern::Grid(_) => 1,
        };
        egui::ComboBox::from_label("Pattern")
            .selected_text(if pattern_kind == 0 { "NoPattern" } else { "Grid" })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut pattern_kind, 0, "NoPattern");
                ui.selectable_value(&mut pattern_kind, 1, "Grid");
            });
        if pattern_kind == 0 {
            pattern = BackgroundPattern::NoPattern;
        } else if !matches!(pattern, BackgroundPattern::Grid(_)) {
            pattern = BackgroundPattern::grid(egui::vec2(50.0, 50.0), 0.0);
        }
        if let BackgroundPattern::Grid(Grid { spacing, angle }) = &mut pattern {
            ui.add(egui::Slider::new(&mut spacing.x, 5.0..=200.0).text("Grid spacing X"));
            ui.add(egui::Slider::new(&mut spacing.y, 5.0..=200.0).text("Grid spacing Y"));
            ui.add(
                egui::Slider::new(angle, 0.0..=std::f32::consts::TAU).text("Grid angle (rad)"),
            );
        }
        style.bg_pattern = Some(pattern);

        let bg_stroke = style
            .bg_pattern_stroke
            .get_or_insert(egui::Stroke::new(1.0, egui::Color32::from_gray(70)));
        ui.add(egui::Slider::new(&mut bg_stroke.width, 0.0..=6.0).text("Pattern stroke width"));
        ui.horizontal(|ui| {
            ui.label("Pattern stroke color");
            ui.color_edit_button_srgba(&mut bg_stroke.color);
        });

        ui.collapsing("Node frame", |ui| {
            let node_frame = style
                .node_frame
                .get_or_insert_with(|| egui::Frame::window(ui.style()));
            ui.horizontal(|ui| {
                ui.label("Fill");
                ui.color_edit_button_srgba(&mut node_frame.fill);
            });
            ui.add(egui::Slider::new(&mut node_frame.stroke.width, 0.0..=8.0).text("Stroke width"));
            ui.horizontal(|ui| {
                ui.label("Stroke color");
                ui.color_edit_button_srgba(&mut node_frame.stroke.color);
            });
            edit_corner_radius(ui, "Corner radius", &mut node_frame.corner_radius);
            edit_margin(ui, "Inner margin", &mut node_frame.inner_margin);
            edit_margin(ui, "Outer margin", &mut node_frame.outer_margin);
        });

        ui.collapsing("Header frame", |ui| {
            let header_frame = style.header_frame.get_or_insert_with(|| {
                let mut f = egui::Frame::window(ui.style());
                f.shadow = egui::epaint::Shadow::NONE;
                f
            });
            ui.horizontal(|ui| {
                ui.label("Fill");
                ui.color_edit_button_srgba(&mut header_frame.fill);
            });
            ui.add(
                egui::Slider::new(&mut header_frame.stroke.width, 0.0..=8.0).text("Stroke width"),
            );
            ui.horizontal(|ui| {
                ui.label("Stroke color");
                ui.color_edit_button_srgba(&mut header_frame.stroke.color);
            });
            edit_corner_radius(ui, "Corner radius", &mut header_frame.corner_radius);
            edit_margin(ui, "Inner margin", &mut header_frame.inner_margin);
            edit_margin(ui, "Outer margin", &mut header_frame.outer_margin);
        });

        ui.collapsing("Background frame", |ui| {
            let bg_frame = style.bg_frame.get_or_insert_with(|| egui::Frame::canvas(ui.style()));
            ui.horizontal(|ui| {
                ui.label("Fill");
                ui.color_edit_button_srgba(&mut bg_frame.fill);
            });
            ui.add(egui::Slider::new(&mut bg_frame.stroke.width, 0.0..=8.0).text("Stroke width"));
            ui.horizontal(|ui| {
                ui.label("Stroke color");
                ui.color_edit_button_srgba(&mut bg_frame.stroke.color);
            });
            edit_corner_radius(ui, "Corner radius", &mut bg_frame.corner_radius);
            edit_margin(ui, "Inner margin", &mut bg_frame.inner_margin);
            edit_margin(ui, "Outer margin", &mut bg_frame.outer_margin);
        });
    });

    ui.separator();
    ui.collapsing("Interaction and zoom", |ui| {
        ui.add(egui::Slider::new(style.min_scale.get_or_insert(0.1), 0.05..=1.0).text("Min scale"));
        ui.add(egui::Slider::new(style.max_scale.get_or_insert(2.0), 1.0..=4.0).text("Max scale"));
        if let (Some(min), Some(max)) = (style.min_scale, style.max_scale)
            && min >= max
        {
            style.max_scale = Some(min + 0.1);
        }
        ui.checkbox(style.centering.get_or_insert(true), "Double-click centering");
        ui.checkbox(
            style.select_rect_contained.get_or_insert(false),
            "Select only fully contained nodes",
        );
        ui.checkbox(
            style.crisp_magnified_text.get_or_insert(false),
            "Crisp magnified text",
        );
    });

    ui.separator();
    ui.collapsing("Selection", |ui| {
        let select_stroke = style
            .select_stoke
            .get_or_insert(egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 160, 255)));
        ui.add(egui::Slider::new(&mut select_stroke.width, 0.0..=8.0).text("Stroke width"));
        ui.horizontal(|ui| {
            ui.label("Stroke color");
            ui.color_edit_button_srgba(&mut select_stroke.color);
        });

        ui.horizontal(|ui| {
            ui.label("Fill");
            let fill = style
                .select_fill
                .get_or_insert(egui::Color32::from_rgba_unmultiplied(80, 160, 255, 48));
            ui.color_edit_button_srgba(fill);
        });

        let select_style = style.select_style.get_or_insert(SelectionStyle {
            margin: ui.spacing().window_margin,
            rounding: ui.visuals().window_corner_radius,
            fill: style.select_fill.unwrap_or(egui::Color32::from_rgba_unmultiplied(
                80, 160, 255, 48,
            )),
            stroke: style.select_stoke.unwrap_or(egui::Stroke::new(
                1.0,
                egui::Color32::from_rgb(80, 160, 255),
            )),
        });
        ui.horizontal(|ui| {
            ui.label("Style fill");
            ui.color_edit_button_srgba(&mut select_style.fill);
        });
        ui.add(egui::Slider::new(&mut select_style.stroke.width, 0.0..=8.0).text("Style stroke"));
        ui.horizontal(|ui| {
            ui.label("Style stroke color");
            ui.color_edit_button_srgba(&mut select_style.stroke.color);
        });
        edit_corner_radius(ui, "Selection rounding", &mut select_style.rounding);
        edit_margin(ui, "Selection margin", &mut select_style.margin);
    });
}
