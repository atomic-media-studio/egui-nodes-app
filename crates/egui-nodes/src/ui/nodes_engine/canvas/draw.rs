//! Drawing of nodes, pins, headers, and bodies — invoked from [`super::scene::show_nodes_canvas`].
//!
//! Contains [`draw_node`] and helpers; selection outline redraws pins on top after the frame stroke.

use std::collections::HashMap;

use egui::{
    Align, Color32, Id, Layout, Modifiers, PointerButton, Pos2, Rect, Sense, StrokeKind, Ui,
    UiBuilder, Vec2, collapsing_header::paint_default_icon, emath::GuiRounding, pos2, vec2,
};
use smallvec::SmallVec;

use super::super::{InPin, InPinId, Node, NodeGraph, NodeId, OutPin, OutPinId};
use super::graph_state::{CanvasState, NodeState, RowHeights};
use super::node_viewer::NodeGraphViewer;
use super::pin::{AnyPin, GraphPin, PinInfo};
use super::style::{CanvasStyle, Heights, NodeLayoutKind, PinPlacement};
use super::wire::WireStyle;

pub(crate) struct DrawNodeResponse {
    pub(crate) node_moved: Option<(NodeId, Vec2)>,
    pub(crate) node_to_top: Option<NodeId>,
    pub(crate) drag_released: bool,
    pub(crate) pin_hovered: Option<AnyPin>,
    pub(crate) final_rect: Rect,
}

pub(crate) struct DrawPinsResponse {
    pub(crate) drag_released: bool,
    pub(crate) pin_hovered: Option<AnyPin>,
    pub(crate) final_rect: Rect,
    pub(crate) new_heights: RowHeights,
}

pub(crate) struct DrawBodyResponse {
    pub(crate) final_rect: Rect,
}

pub(crate) struct PinResponse {
    pub(crate) pos: Pos2,
    /// Last drawn pin shape rect (includes hover scale).
    pub(crate) draw_rect: Rect,
    pub(crate) pin_info: PinInfo,
    pub(crate) wire_color: Color32,
    pub(crate) wire_style: WireStyle,
}
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
fn draw_inputs<T, V>(
    node_graph: &mut NodeGraph<T>,
    viewer: &mut V,
    node: NodeId,
    inputs: &[InPin],
    pin_size: f32,
    style: &CanvasStyle,
    node_ui: &mut Ui,
    inputs_rect: Rect,
    payload_clip_rect: Rect,
    input_x: f32,
    min_pin_y_top: f32,
    min_pin_y_bottom: f32,
    input_spacing: Option<f32>,
    canvas_state: &mut CanvasState,
    modifiers: Modifiers,
    input_positions: &mut HashMap<InPinId, PinResponse>,
    heights: Heights,
) -> DrawPinsResponse
where
    V: NodeGraphViewer<T>,
{
    let mut drag_released = false;
    let mut pin_hovered = None;

    // Input pins on the left.
    let mut inputs_ui = node_ui.new_child(
        UiBuilder::new()
            .max_rect(inputs_rect.round_ui())
            .layout(Layout::top_down(Align::Min))
            .id_salt("inputs"),
    );

    let graph_clip_rect = node_ui.clip_rect();
    inputs_ui.shrink_clip_rect(payload_clip_rect);

    let pin_layout = Layout::left_to_right(Align::Min);
    let mut new_heights = SmallVec::with_capacity(inputs.len());

    for in_pin in inputs {
        // Show input pin.
        let cursor = inputs_ui.cursor();
        let (height, height_outer) = heights.get(in_pin.id.input);

        let margin = (height_outer - height) / 2.0;
        let outer_rect = cursor.with_max_y(cursor.top() + height_outer);
        let inner_rect = outer_rect.shrink2(vec2(0.0, margin));

        let builder = UiBuilder::new().layout(pin_layout).max_rect(inner_rect);

        inputs_ui.scope_builder(builder, |pin_ui| {
            if let Some(input_spacing) = input_spacing {
                let min = pin_ui.next_widget_position();
                pin_ui.advance_cursor_after_rect(Rect::from_min_size(
                    min,
                    vec2(input_spacing, pin_size),
                ));
            }

            let y0 = pin_ui.max_rect().min.y;
            let y1 = pin_ui.max_rect().max.y;

            // Show input content
            let graph_pin = viewer.show_input(in_pin, pin_ui, node_graph);
            if !node_graph.nodes.contains(node.0) {
                // If removed
                return;
            }

            let pin_rect = graph_pin.pin_rect(
                input_x,
                min_pin_y_top.max(y0),
                min_pin_y_bottom.max(y1),
                pin_size,
            );

            // Interact with pin shape.
            pin_ui.set_clip_rect(graph_clip_rect);

            let r = pin_ui.interact(pin_rect, pin_ui.next_auto_id(), Sense::click_and_drag());

            pin_ui.skip_ahead_auto_ids(1);

            if r.clicked_by(PointerButton::Secondary) {
                if canvas_state.has_new_wires() {
                    canvas_state.remove_new_wire_in(in_pin.id);
                } else {
                    viewer.drop_inputs(in_pin, node_graph);
                    if !node_graph.nodes.contains(node.0) {
                        // If removed
                        return;
                    }
                }
            }
            if r.drag_started_by(PointerButton::Primary) {
                if modifiers.command {
                    canvas_state.start_new_wires_out(&in_pin.remotes);
                    if !modifiers.shift {
                        node_graph.drop_inputs(in_pin.id);
                        if !node_graph.nodes.contains(node.0) {
                            // If removed
                            return;
                        }
                    }
                } else {
                    canvas_state.start_new_wire_in(in_pin.id);
                }
            }

            if r.drag_stopped() {
                drag_released = true;
            }

            let mut visual_pin_rect = r.rect;

            if r.contains_pointer() {
                if canvas_state.has_new_wires_in() {
                    if modifiers.shift && !modifiers.command {
                        canvas_state.add_new_wire_in(in_pin.id);
                    }
                    if !modifiers.shift && modifiers.command {
                        canvas_state.remove_new_wire_in(in_pin.id);
                    }
                }
                pin_hovered = Some(AnyPin::In(in_pin.id));
                visual_pin_rect = visual_pin_rect.scale_from_center(1.2);
            }

            let wire_info =
                graph_pin.draw(style, pin_ui.style(), visual_pin_rect, pin_ui.painter());

            input_positions.insert(
                in_pin.id,
                PinResponse {
                    pos: r.rect.center(),
                    draw_rect: visual_pin_rect,
                    pin_info: graph_pin,
                    wire_color: wire_info.color,
                    wire_style: wire_info.style,
                },
            );

            new_heights.push(pin_ui.min_rect().height());

            pin_ui.expand_to_include_y(outer_rect.bottom());
        });
    }

    let final_rect = inputs_ui.min_rect();
    node_ui.expand_to_include_rect(final_rect.intersect(payload_clip_rect));

    DrawPinsResponse {
        drag_released,
        pin_hovered,
        final_rect,
        new_heights,
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
fn draw_outputs<T, V>(
    node_graph: &mut NodeGraph<T>,
    viewer: &mut V,
    node: NodeId,
    outputs: &[OutPin],
    pin_size: f32,
    style: &CanvasStyle,
    node_ui: &mut Ui,
    outputs_rect: Rect,
    payload_clip_rect: Rect,
    output_x: f32,
    min_pin_y_top: f32,
    min_pin_y_bottom: f32,
    output_spacing: Option<f32>,
    canvas_state: &mut CanvasState,
    modifiers: Modifiers,
    output_positions: &mut HashMap<OutPinId, PinResponse>,
    heights: Heights,
) -> DrawPinsResponse
where
    V: NodeGraphViewer<T>,
{
    let mut drag_released = false;
    let mut pin_hovered = None;

    let mut outputs_ui = node_ui.new_child(
        UiBuilder::new()
            .max_rect(outputs_rect.round_ui())
            .layout(Layout::top_down(Align::Max))
            .id_salt("outputs"),
    );

    let graph_clip_rect = node_ui.clip_rect();
    outputs_ui.shrink_clip_rect(payload_clip_rect);

    let pin_layout = Layout::right_to_left(Align::Min);
    let mut new_heights = SmallVec::with_capacity(outputs.len());

    // Output pins on the right.
    for out_pin in outputs {
        // Show output pin.
        let cursor = outputs_ui.cursor();
        let (height, height_outer) = heights.get(out_pin.id.output);

        let margin = (height_outer - height) / 2.0;
        let outer_rect = cursor.with_max_y(cursor.top() + height_outer);
        let inner_rect = outer_rect.shrink2(vec2(0.0, margin));

        let builder = UiBuilder::new().layout(pin_layout).max_rect(inner_rect);

        outputs_ui.scope_builder(builder, |pin_ui| {
            // Allocate space for pin shape.
            if let Some(output_spacing) = output_spacing {
                let min = pin_ui.next_widget_position();
                pin_ui.advance_cursor_after_rect(Rect::from_min_size(
                    min,
                    vec2(output_spacing, pin_size),
                ));
            }

            let y0 = pin_ui.max_rect().min.y;
            let y1 = pin_ui.max_rect().max.y;

            // Show output content
            let graph_pin = viewer.show_output(out_pin, pin_ui, node_graph);
            if !node_graph.nodes.contains(node.0) {
                // If removed
                return;
            }

            let pin_rect = graph_pin.pin_rect(
                output_x,
                min_pin_y_top.max(y0),
                min_pin_y_bottom.max(y1),
                pin_size,
            );

            pin_ui.set_clip_rect(graph_clip_rect);

            let r = pin_ui.interact(pin_rect, pin_ui.next_auto_id(), Sense::click_and_drag());

            pin_ui.skip_ahead_auto_ids(1);

            if r.clicked_by(PointerButton::Secondary) {
                if canvas_state.has_new_wires() {
                    canvas_state.remove_new_wire_out(out_pin.id);
                } else {
                    viewer.drop_outputs(out_pin, node_graph);
                    if !node_graph.nodes.contains(node.0) {
                        // If removed
                        return;
                    }
                }
            }
            if r.drag_started_by(PointerButton::Primary) {
                if modifiers.command {
                    canvas_state.start_new_wires_in(&out_pin.remotes);

                    if !modifiers.shift {
                        node_graph.drop_outputs(out_pin.id);
                        if !node_graph.nodes.contains(node.0) {
                            // If removed
                            return;
                        }
                    }
                } else {
                    canvas_state.start_new_wire_out(out_pin.id);
                }
            }

            if r.drag_stopped() {
                drag_released = true;
            }

            let mut visual_pin_rect = r.rect;

            if r.contains_pointer() {
                if canvas_state.has_new_wires_out() {
                    if modifiers.shift && !modifiers.command {
                        canvas_state.add_new_wire_out(out_pin.id);
                    }
                    if !modifiers.shift && modifiers.command {
                        canvas_state.remove_new_wire_out(out_pin.id);
                    }
                }
                pin_hovered = Some(AnyPin::Out(out_pin.id));
                visual_pin_rect = visual_pin_rect.scale_from_center(1.2);
            }

            let wire_info =
                graph_pin.draw(style, pin_ui.style(), visual_pin_rect, pin_ui.painter());

            output_positions.insert(
                out_pin.id,
                PinResponse {
                    pos: r.rect.center(),
                    draw_rect: visual_pin_rect,
                    pin_info: graph_pin,
                    wire_color: wire_info.color,
                    wire_style: wire_info.style,
                },
            );

            new_heights.push(pin_ui.min_rect().height());

            pin_ui.expand_to_include_y(outer_rect.bottom());
        });
    }
    let final_rect = outputs_ui.min_rect();
    node_ui.expand_to_include_rect(final_rect.intersect(payload_clip_rect));

    DrawPinsResponse {
        drag_released,
        pin_hovered,
        final_rect,
        new_heights,
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_body<T, V>(
    node_graph: &mut NodeGraph<T>,
    viewer: &mut V,
    node: NodeId,
    inputs: &[InPin],
    outputs: &[OutPin],
    ui: &mut Ui,
    body_rect: Rect,
    payload_clip_rect: Rect,
    _canvas_state: &CanvasState,
) -> DrawBodyResponse
where
    V: NodeGraphViewer<T>,
{
    let mut body_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(body_rect.round_ui())
            .layout(Layout::left_to_right(Align::Min))
            .id_salt("body"),
    );

    body_ui.shrink_clip_rect(payload_clip_rect);

    viewer.show_body(node, inputs, outputs, &mut body_ui, node_graph);

    let final_rect = body_ui.min_rect();
    ui.expand_to_include_rect(final_rect.intersect(payload_clip_rect));
    // node_state.set_body_width(body_size.x);

    DrawBodyResponse { final_rect }
}

//First step for split big function to parts
/// Draw one node. Return Pins info
#[inline]
#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_node<T, V>(
    node_graph: &mut NodeGraph<T>,
    ui: &mut Ui,
    node: NodeId,
    viewer: &mut V,
    canvas_state: &mut CanvasState,
    style: &CanvasStyle,
    canvas_id: Id,
    input_positions: &mut HashMap<InPinId, PinResponse>,
    modifiers: Modifiers,
    output_positions: &mut HashMap<OutPinId, PinResponse>,
) -> Option<DrawNodeResponse>
where
    V: NodeGraphViewer<T>,
{
    let Node {
        pos,
        open,
        ref value,
    } = node_graph.nodes[node.0];

    // Collect pins
    let inputs_count = viewer.inputs(value);
    let outputs_count = viewer.outputs(value);

    let inputs = (0..inputs_count)
        .map(|idx| InPin::new(node_graph, InPinId { node, input: idx }))
        .collect::<Vec<_>>();

    let outputs = (0..outputs_count)
        .map(|idx| OutPin::new(node_graph, OutPinId { node, output: idx }))
        .collect::<Vec<_>>();

    let node_pos = pos.round_ui();

    // Generate persistent id for the node.
    let node_id = canvas_id.with(("node_graph-node", node));

    let openness = ui.ctx().animate_bool(node_id, open);

    let mut node_state = NodeState::load(ui.ctx(), node_id, ui.spacing());

    let node_rect = node_state.node_rect(node_pos, openness);

    let mut node_to_top = None;
    let mut node_moved = None;
    let mut drag_released = false;
    let mut pin_hovered = None;

    let node_frame = viewer.node_frame(
        style.get_node_frame(ui.style()),
        node,
        &inputs,
        &outputs,
        node_graph,
    );

    let header_frame = viewer.header_frame(
        style.get_header_frame(ui.style()),
        node,
        &inputs,
        &outputs,
        node_graph,
    );

    // Rect for node + frame margin.
    let node_frame_rect = node_rect + node_frame.total_margin();

    // Size of the pin.
    // Side of the square or diameter of the circle.
    let pin_size = style.get_pin_size(ui.style()).max(0.0);

    let pin_placement = style.get_pin_placement();

    let header_drag_space = style.get_header_drag_space(ui.style()).max(Vec2::ZERO);

    // Interact with node frame.
    let r = ui.interact(
        node_frame_rect,
        node_id.with("frame"),
        Sense::click_and_drag(),
    );

    if !modifiers.shift && !modifiers.command && r.dragged_by(PointerButton::Primary) {
        // Dragging does not emit `clicked_by` until release; select as soon as we move so the
        // outline matches. If this node is already part of a multi-selection, keep the set so
        // group moves still work.
        if !canvas_state.selected_nodes().contains(&node) {
            canvas_state.select_one_node(true, node);
        }
        node_moved = Some((node, r.drag_delta()));
    }

    if r.clicked_by(PointerButton::Primary) {
        canvas_state.select_one_node(true, node);
    }

    if r.clicked() || r.dragged() {
        node_to_top = Some(node);
    }

    if viewer.has_node_menu(&node_graph.nodes[node.0].value) {
        r.context_menu(|ui| {
            viewer.show_node_menu(node, &inputs, &outputs, ui, node_graph);
        });
    }

    if !node_graph.nodes.contains(node.0) {
        node_state.clear(ui.ctx());
        // If removed
        return None;
    }

    if viewer.has_on_hover_popup(&node_graph.nodes[node.0].value) {
        r.on_hover_ui_at_pointer(|ui| {
            viewer.show_on_hover_popup(node, &inputs, &outputs, ui, node_graph);
        });
    }

    if !node_graph.nodes.contains(node.0) {
        node_state.clear(ui.ctx());
        // If removed
        return None;
    }

    let node_ui = &mut ui.new_child(
        UiBuilder::new()
            .max_rect(node_frame_rect.round_ui())
            .layout(Layout::top_down(Align::Center))
            .id_salt(node_id),
    );

    let mut new_pins_size = Vec2::ZERO;

    let r = node_frame.show(node_ui, |ui| {
        if viewer.has_node_style(node, &inputs, &outputs, node_graph) {
            viewer.apply_node_style(ui.style_mut(), node, &inputs, &outputs, node_graph);
        }

        // Input pins' center side by X axis.
        let input_x = match pin_placement {
            PinPlacement::Inside => pin_size.mul_add(
                0.5,
                node_frame_rect.left() + node_frame.inner_margin.leftf(),
            ),
            PinPlacement::Edge => node_frame_rect.left(),
            PinPlacement::Outside { margin } => {
                pin_size.mul_add(-0.5, node_frame_rect.left() - margin)
            }
        };

        // Input pins' spacing required.
        let input_spacing = match pin_placement {
            PinPlacement::Inside => Some(pin_size),
            PinPlacement::Edge => Some(
                pin_size
                    .mul_add(0.5, -node_frame.inner_margin.leftf())
                    .max(0.0),
            ),
            PinPlacement::Outside { .. } => None,
        };

        // Output pins' center side by X axis.
        let output_x = match pin_placement {
            PinPlacement::Inside => pin_size.mul_add(
                -0.5,
                node_frame_rect.right() - node_frame.inner_margin.rightf(),
            ),
            PinPlacement::Edge => node_frame_rect.right(),
            PinPlacement::Outside { margin } => {
                pin_size.mul_add(0.5, node_frame_rect.right() + margin)
            }
        };

        // Output pins' spacing required.
        let output_spacing = match pin_placement {
            PinPlacement::Inside => Some(pin_size),
            PinPlacement::Edge => Some(
                pin_size
                    .mul_add(0.5, -node_frame.inner_margin.rightf())
                    .max(0.0),
            ),
            PinPlacement::Outside { .. } => None,
        };

        // Input/output pin block

        if (openness < 1.0 && open) || (openness > 0.0 && !open) {
            ui.ctx().request_repaint();
        }

        // Pins are placed under the header and must not go outside of the header frame.
        let payload_rect = Rect::from_min_max(
            pos2(
                node_rect.min.x,
                node_rect.min.y
                    + node_state.header_height()
                    + header_frame.total_margin().bottom
                    + ui.spacing().item_spacing.y
                    - node_state.payload_offset(openness),
            ),
            node_rect.max,
        );

        let node_layout =
            viewer.node_layout(style.get_node_layout(), node, &inputs, &outputs, node_graph);

        let payload_clip_rect =
            Rect::from_min_max(node_rect.min, pos2(node_rect.max.x, f32::INFINITY));

        let pins_rect = match node_layout.kind {
            NodeLayoutKind::Coil => {
                // Show input pins.
                let r = draw_inputs(
                    node_graph,
                    viewer,
                    node,
                    &inputs,
                    pin_size,
                    style,
                    ui,
                    payload_rect,
                    payload_clip_rect,
                    input_x,
                    node_rect.min.y,
                    node_rect.min.y + node_state.header_height(),
                    input_spacing,
                    canvas_state,
                    modifiers,
                    input_positions,
                    node_layout.input_heights(&node_state),
                );

                let new_input_heights = r.new_heights;

                drag_released |= r.drag_released;

                if r.pin_hovered.is_some() {
                    pin_hovered = r.pin_hovered;
                }

                let inputs_rect = r.final_rect;
                let inputs_size = inputs_rect.size();

                if !node_graph.nodes.contains(node.0) {
                    // If removed
                    return;
                }

                // Show output pins.

                let r = draw_outputs(
                    node_graph,
                    viewer,
                    node,
                    &outputs,
                    pin_size,
                    style,
                    ui,
                    payload_rect,
                    payload_clip_rect,
                    output_x,
                    node_rect.min.y,
                    node_rect.min.y + node_state.header_height(),
                    output_spacing,
                    canvas_state,
                    modifiers,
                    output_positions,
                    node_layout.output_heights(&node_state),
                );

                let new_output_heights = r.new_heights;

                drag_released |= r.drag_released;

                if r.pin_hovered.is_some() {
                    pin_hovered = r.pin_hovered;
                }

                let outputs_rect = r.final_rect;
                let outputs_size = outputs_rect.size();

                if !node_graph.nodes.contains(node.0) {
                    // If removed
                    return;
                }

                node_state.set_input_heights(new_input_heights);
                node_state.set_output_heights(new_output_heights);

                new_pins_size = vec2(
                    inputs_size.x + outputs_size.x + ui.spacing().item_spacing.x,
                    f32::max(inputs_size.y, outputs_size.y),
                );

                let mut pins_rect = inputs_rect.union(outputs_rect);

                // Show body if there's one.
                if viewer.has_body(&node_graph.nodes.get(node.0).unwrap().value) {
                    let body_rect = Rect::from_min_max(
                        pos2(
                            inputs_rect.right() + ui.spacing().item_spacing.x,
                            payload_rect.top(),
                        ),
                        pos2(
                            outputs_rect.left() - ui.spacing().item_spacing.x,
                            payload_rect.bottom(),
                        ),
                    );

                    let r = draw_body(
                        node_graph,
                        viewer,
                        node,
                        &inputs,
                        &outputs,
                        ui,
                        body_rect,
                        payload_clip_rect,
                        canvas_state,
                    );

                    new_pins_size.x += r.final_rect.width() + ui.spacing().item_spacing.x;
                    new_pins_size.y = f32::max(new_pins_size.y, r.final_rect.height());

                    pins_rect = pins_rect.union(body_rect);

                    if !node_graph.nodes.contains(node.0) {
                        // If removed
                        return;
                    }
                }

                pins_rect
            }
            NodeLayoutKind::Sandwich => {
                // Show input pins.

                let r = draw_inputs(
                    node_graph,
                    viewer,
                    node,
                    &inputs,
                    pin_size,
                    style,
                    ui,
                    payload_rect,
                    payload_clip_rect,
                    input_x,
                    node_rect.min.y,
                    node_rect.min.y + node_state.header_height(),
                    input_spacing,
                    canvas_state,
                    modifiers,
                    input_positions,
                    node_layout.input_heights(&node_state),
                );

                let new_input_heights = r.new_heights;

                drag_released |= r.drag_released;

                if r.pin_hovered.is_some() {
                    pin_hovered = r.pin_hovered;
                }

                let inputs_rect = r.final_rect;

                new_pins_size = inputs_rect.size();

                let mut next_y = inputs_rect.bottom() + ui.spacing().item_spacing.y;

                if !node_graph.nodes.contains(node.0) {
                    // If removed
                    return;
                }

                let mut pins_rect = inputs_rect;

                // Show body if there's one.
                if viewer.has_body(&node_graph.nodes.get(node.0).unwrap().value) {
                    let body_rect = payload_rect.intersect(Rect::everything_below(next_y));

                    let r = draw_body(
                        node_graph,
                        viewer,
                        node,
                        &inputs,
                        &outputs,
                        ui,
                        body_rect,
                        payload_clip_rect,
                        canvas_state,
                    );

                    let body_rect = r.final_rect;

                    new_pins_size.x = f32::max(new_pins_size.x, body_rect.width());
                    new_pins_size.y += body_rect.height() + ui.spacing().item_spacing.y;

                    if !node_graph.nodes.contains(node.0) {
                        // If removed
                        return;
                    }

                    pins_rect = pins_rect.union(body_rect);
                    next_y = body_rect.bottom() + ui.spacing().item_spacing.y;
                }

                // Show output pins.

                let outputs_rect = payload_rect.intersect(Rect::everything_below(next_y));

                let r = draw_outputs(
                    node_graph,
                    viewer,
                    node,
                    &outputs,
                    pin_size,
                    style,
                    ui,
                    outputs_rect,
                    payload_clip_rect,
                    output_x,
                    node_rect.min.y,
                    node_rect.min.y + node_state.header_height(),
                    output_spacing,
                    canvas_state,
                    modifiers,
                    output_positions,
                    node_layout.output_heights(&node_state),
                );

                let new_output_heights = r.new_heights;

                drag_released |= r.drag_released;

                if r.pin_hovered.is_some() {
                    pin_hovered = r.pin_hovered;
                }

                let outputs_rect = r.final_rect;

                if !node_graph.nodes.contains(node.0) {
                    // If removed
                    return;
                }

                node_state.set_input_heights(new_input_heights);
                node_state.set_output_heights(new_output_heights);

                new_pins_size.x = f32::max(new_pins_size.x, outputs_rect.width());
                new_pins_size.y += outputs_rect.height() + ui.spacing().item_spacing.y;

                pins_rect = pins_rect.union(outputs_rect);

                pins_rect
            }
            NodeLayoutKind::FlippedSandwich => {
                // Show input pins.

                let outputs_rect = payload_rect;
                let r = draw_outputs(
                    node_graph,
                    viewer,
                    node,
                    &outputs,
                    pin_size,
                    style,
                    ui,
                    outputs_rect,
                    payload_clip_rect,
                    output_x,
                    node_rect.min.y,
                    node_rect.min.y + node_state.header_height(),
                    output_spacing,
                    canvas_state,
                    modifiers,
                    output_positions,
                    node_layout.output_heights(&node_state),
                );

                let new_output_heights = r.new_heights;

                drag_released |= r.drag_released;

                if r.pin_hovered.is_some() {
                    pin_hovered = r.pin_hovered;
                }

                let outputs_rect = r.final_rect;

                new_pins_size = outputs_rect.size();

                let mut next_y = outputs_rect.bottom() + ui.spacing().item_spacing.y;

                if !node_graph.nodes.contains(node.0) {
                    // If removed
                    return;
                }

                let mut pins_rect = outputs_rect;

                // Show body if there's one.
                if viewer.has_body(&node_graph.nodes.get(node.0).unwrap().value) {
                    let body_rect = payload_rect.intersect(Rect::everything_below(next_y));

                    let r = draw_body(
                        node_graph,
                        viewer,
                        node,
                        &inputs,
                        &outputs,
                        ui,
                        body_rect,
                        payload_clip_rect,
                        canvas_state,
                    );

                    let body_rect = r.final_rect;

                    new_pins_size.x = f32::max(new_pins_size.x, body_rect.width());
                    new_pins_size.y += body_rect.height() + ui.spacing().item_spacing.y;

                    if !node_graph.nodes.contains(node.0) {
                        // If removed
                        return;
                    }

                    pins_rect = pins_rect.union(body_rect);
                    next_y = body_rect.bottom() + ui.spacing().item_spacing.y;
                }

                // Show output pins.

                let inputs_rect = payload_rect.intersect(Rect::everything_below(next_y));

                let r = draw_inputs(
                    node_graph,
                    viewer,
                    node,
                    &inputs,
                    pin_size,
                    style,
                    ui,
                    inputs_rect,
                    payload_clip_rect,
                    input_x,
                    node_rect.min.y,
                    node_rect.min.y + node_state.header_height(),
                    input_spacing,
                    canvas_state,
                    modifiers,
                    input_positions,
                    node_layout.input_heights(&node_state),
                );

                let new_input_heights = r.new_heights;

                drag_released |= r.drag_released;

                if r.pin_hovered.is_some() {
                    pin_hovered = r.pin_hovered;
                }

                let inputs_rect = r.final_rect;

                if !node_graph.nodes.contains(node.0) {
                    // If removed
                    return;
                }

                node_state.set_input_heights(new_input_heights);
                node_state.set_output_heights(new_output_heights);

                new_pins_size.x = f32::max(new_pins_size.x, inputs_rect.width());
                new_pins_size.y += inputs_rect.height() + ui.spacing().item_spacing.y;

                pins_rect = pins_rect.union(inputs_rect);

                pins_rect
            }
        };

        if viewer.has_footer(&node_graph.nodes[node.0].value) {
            let footer_rect = Rect::from_min_max(
                pos2(
                    node_rect.left(),
                    pins_rect.bottom() + ui.spacing().item_spacing.y,
                ),
                pos2(node_rect.right(), node_rect.bottom()),
            );

            let mut footer_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(footer_rect.round_ui())
                    .layout(Layout::left_to_right(Align::Min))
                    .id_salt("footer"),
            );
            footer_ui.shrink_clip_rect(payload_clip_rect);

            viewer.show_footer(node, &inputs, &outputs, &mut footer_ui, node_graph);

            let final_rect = footer_ui.min_rect();
            ui.expand_to_include_rect(final_rect.intersect(payload_clip_rect));
            let footer_size = final_rect.size();

            new_pins_size.x = f32::max(new_pins_size.x, footer_size.x);
            new_pins_size.y += footer_size.y + ui.spacing().item_spacing.y;

            if !node_graph.nodes.contains(node.0) {
                // If removed
                return;
            }
        }

        // Render header frame.
        let mut header_rect = Rect::NAN;

        let mut header_frame_rect = Rect::NAN; //node_rect + header_frame.total_margin();

        // Show node's header
        let header_ui: &mut Ui = &mut ui.new_child(
            UiBuilder::new()
                .max_rect(node_rect.round_ui() + header_frame.total_margin())
                .layout(Layout::top_down(Align::Center))
                .id_salt("header"),
        );

        header_frame.show(header_ui, |ui: &mut Ui| {
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                if style.get_collapsible() {
                    let (_, r) = ui.allocate_exact_size(
                        vec2(ui.spacing().icon_width, ui.spacing().icon_width),
                        Sense::click(),
                    );
                    paint_default_icon(ui, openness, &r);

                    if r.clicked_by(PointerButton::Primary) {
                        // Toggle node's openness.
                        node_graph.open_node(node, !open);
                    }
                }

                ui.allocate_exact_size(header_drag_space, Sense::hover());

                viewer.show_header(node, &inputs, &outputs, ui, node_graph);

                header_rect = ui.min_rect();
            });

            header_frame_rect = header_rect + header_frame.total_margin();

            ui.advance_cursor_after_rect(Rect::from_min_max(
                header_rect.min,
                pos2(
                    f32::max(header_rect.max.x, node_rect.max.x),
                    header_rect.min.y,
                ),
            ));
        });

        ui.expand_to_include_rect(header_rect);
        let header_size = header_rect.size();
        node_state.set_header_height(header_size.y);

        node_state.set_size(vec2(
            f32::max(header_size.x, new_pins_size.x),
            header_size.y
                + header_frame.total_margin().bottom
                + ui.spacing().item_spacing.y
                + new_pins_size.y,
        ));
    });

    // Draw selection after header and body so the outline sits on top (header was covering it).
    // Single outline around the full node; default margin is zero so it is not offset outward.
    if canvas_state.selected_nodes().contains(&node) {
        let select_style = style.get_select_style(ui.style());
        let select_rect = node_frame_rect + select_style.margin;
        ui.painter().rect(
            select_rect,
            select_style.rounding,
            select_style.fill,
            select_style.stroke,
            StrokeKind::Inside,
        );
        // Selection was painted after pins; draw pins again on top of the outline.
        let painter = ui.painter();
        let egui_style = ui.style();
        for (id, pr) in input_positions.iter() {
            if id.node != node {
                continue;
            }
            let _ = pr.pin_info.draw(style, egui_style, pr.draw_rect, painter);
        }
        for (id, pr) in output_positions.iter() {
            if id.node != node {
                continue;
            }
            let _ = pr.pin_info.draw(style, egui_style, pr.draw_rect, painter);
        }
    }

    if !node_graph.nodes.contains(node.0) {
        ui.ctx().request_repaint();
        node_state.clear(ui.ctx());
        // If removed
        return None;
    }

    viewer.final_node_rect(node, r.response.rect, ui, node_graph);

    node_state.store(ui.ctx());
    Some(DrawNodeResponse {
        node_moved,
        node_to_top,
        drag_released,
        pin_hovered,
        final_rect: r.response.rect,
    })
}
