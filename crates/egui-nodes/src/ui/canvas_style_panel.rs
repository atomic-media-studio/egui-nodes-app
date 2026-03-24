//! Optional egui UI for editing a [`CanvasStyle`] (used by the playground side panel).

use egui::Ui;

use crate::ui::nodes_canvas::{
    BackgroundPattern, GridRenderMode, NodeLayout, PinPlacement, SelectionStyle,
    CanvasStyle, WireLayer, WireStyle,
};

fn edit_margin(ui: &mut Ui, label: &str, margin: &mut egui::Margin) {
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

fn edit_corner_radius(ui: &mut Ui, label: &str, radius: &mut egui::CornerRadius) {
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

/// Node frame corners as **T**op/**B**ottom rows × **L**eft/**R**ight columns (nw, ne, sw, se).
fn edit_node_corner_radius_grid_tb_lr(
    ui: &mut Ui,
    grid_id: egui::Id,
    label: &str,
    radius: &mut egui::CornerRadius,
) {
    ui.label(label);
    egui::Grid::new(grid_id)
        .num_columns(3)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            ui.label("");
            ui.label("L");
            ui.label("R");
            ui.end_row();
            ui.label("T");
            ui.add(egui::DragValue::new(&mut radius.nw).range(0..=64));
            ui.add(egui::DragValue::new(&mut radius.ne).range(0..=64));
            ui.end_row();
            ui.label("B");
            ui.add(egui::DragValue::new(&mut radius.sw).range(0..=64));
            ui.add(egui::DragValue::new(&mut radius.se).range(0..=64));
            ui.end_row();
        });
}

/// Full inspector for [`CanvasStyle`] (collapsible sections: layout, pins, wires, background, interaction, selection).
pub fn canvas_style_controls_ui(ui: &mut Ui, style: &mut CanvasStyle) {
    // Materialize `node_frame` before any section runs so expanding "Node layout" does not flip
    // `None` → `Some` on first open (which used to change corner radius vs implicit chrome).
    style.node_frame.get_or_insert_with(|| {
        let mut f = egui::Frame::window(ui.style());
        f.corner_radius = egui::CornerRadius::same(4);
        f
    });

    ui.collapsing("Node layout", |ui| {
        let mut layout = style.node_layout.unwrap_or(NodeLayout::coil());
        ui.add(egui::Slider::new(&mut layout.min_pin_row_height, 0.0..=60.0).text("Min pin row"));
        style.node_layout = Some(layout);
        ui.checkbox(style.collapsible.get_or_insert(true), "Collapsible nodes");
        let header_drag_space = style.header_drag_space.get_or_insert(egui::vec2(16.0, 16.0));
        ui.add(egui::Slider::new(&mut header_drag_space.x, 0.0..=120.0).text("Header drag X"));
        ui.add(egui::Slider::new(&mut header_drag_space.y, 0.0..=120.0).text("Header drag Y"));

        let node_frame = style
            .node_frame
            .get_or_insert_with(|| egui::Frame::window(ui.style()));
        edit_node_corner_radius_grid_tb_lr(
            ui,
            ui.make_persistent_id("node_layout_node_corner_radius"),
            "Node corner radius (T/B × L/R)",
            &mut node_frame.corner_radius,
        );
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
            let color = style
                .pin_fill
                .get_or_insert(egui::Color32::from_rgba_unmultiplied(70, 70, 70, 255));
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
        ui.add(egui::Slider::new(style.wire_width.get_or_insert(3.0), 0.2..=10.0).text("Width"));
        ui.add(
            egui::Slider::new(style.wire_frame_size.get_or_insert(32.0), 4.0..=120.0)
                .text("Frame size"),
        );
        ui.add(
            egui::Slider::new(style.wire_smoothness.get_or_insert(0.0), 0.0..=10.0)
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
            pattern = BackgroundPattern::new();
        }
        if let BackgroundPattern::Grid(g) = &mut pattern {
            ui.add(egui::Slider::new(&mut g.spacing.x, 5.0..=200.0).text("Grid spacing X"));
            ui.add(egui::Slider::new(&mut g.spacing.y, 5.0..=200.0).text("Grid spacing Y"));
            ui.add(
                egui::Slider::new(&mut g.angle, 0.0..=std::f32::consts::TAU).text("Grid angle (rad)"),
            );
            egui::ComboBox::from_label("Grid look")
                .selected_text(match g.mode {
                    GridRenderMode::Lines => "Lines",
                    GridRenderMode::Dots => "Dots (filled)",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut g.mode, GridRenderMode::Lines, "Lines");
                    ui.selectable_value(&mut g.mode, GridRenderMode::Dots, "Dots (filled)");
                });
            ui.add(egui::Slider::new(&mut g.phase.x, -200.0..=200.0).text("Phase X"));
            ui.add(egui::Slider::new(&mut g.phase.y, -200.0..=200.0).text("Phase Y"));
            if g.mode == GridRenderMode::Dots {
                ui.add(egui::Slider::new(&mut g.dot_radius, 0.5..=12.0).text("Dot radius"));
            }
            ui.horizontal(|ui| {
                ui.label("Grid color (override)");
                let mut use_custom = g.color.is_some();
                ui.checkbox(&mut use_custom, "Custom");
                if use_custom {
                    let c = g.color.get_or_insert(egui::Color32::from_gray(120));
                    ui.color_edit_button_srgba(c);
                } else {
                    g.color = None;
                }
            });
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
        ui.add(egui::Slider::new(style.min_scale.get_or_insert(1.0), 0.05..=1.0).text("Min scale"));
        ui.add(egui::Slider::new(style.max_scale.get_or_insert(1.10), 1.0..=4.0).text("Max scale"));
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
            .get_or_insert(egui::Stroke::new(2.0, egui::Color32::WHITE));
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
            margin: egui::Margin::same(2),
            rounding: egui::CornerRadius::same(4),
            fill: egui::Color32::TRANSPARENT,
            stroke: style.select_stoke.unwrap_or(egui::Stroke::new(2.0, egui::Color32::WHITE)),
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
