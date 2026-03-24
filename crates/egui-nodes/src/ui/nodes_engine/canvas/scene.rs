//! Scene: pan/zoom, marquee selection, wire rendering, and [`NodesCanvas`].

use std::collections::HashMap;
use std::hash::Hash;

use egui::emath::GuiRounding;
use egui::{
    Color32, Context, DragPanButtons, Id, LayerId, PointerButton, Rect, Scene, Sense, Shape,
    Stroke, StrokeKind, Ui, UiBuilder, UiKind, UiStackInfo, Vec2, response::Flags,
};
use egui_scale::EguiScale;

use super::super::{InPin, NodeGraph, NodeId, OutPin};
use super::draw::draw_node;
use super::graph_state::{CanvasState, NewWires};
use super::node_viewer::NodeGraphViewer;
use super::pin::{AnyPin, AnyPins};
use super::style::CanvasStyle;
use super::transform::clamp_scale;
use super::wire::{WireId, WireLayer, draw_wire, hit_wire, pick_wire_style};

const fn mix_colors(a: Color32, b: Color32) -> Color32 {
    #![allow(clippy::cast_possible_truncation)]

    Color32::from_rgba_premultiplied(
        u8::midpoint(a.r(), b.r()),
        u8::midpoint(a.g(), b.g()),
        u8::midpoint(a.b(), b.b()),
        u8::midpoint(a.a(), b.a()),
    )
}

/// Widget to display [`NodeGraph`] graph in [`Ui`].
#[derive(Clone, Copy, Debug)]
pub struct NodesCanvas {
    id_salt: Id,
    id: Option<Id>,
    style: CanvasStyle,
    min_size: Vec2,
    max_size: Vec2,
}

impl Default for NodesCanvas {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl NodesCanvas {
    /// Returns new [`NodesCanvas`] with default parameters.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        NodesCanvas {
            id_salt: Id::new(":node_graph:"),
            id: None,
            style: CanvasStyle::new(),
            min_size: Vec2::ZERO,
            max_size: Vec2::INFINITY,
        }
    }

    /// Assign an explicit and globally unique [`Id`].
    ///
    /// Use this if you want to persist the state of the widget
    /// when it changes position in the widget hierarchy.
    ///
    /// Prefer using [`NodesCanvas::id_salt`] otherwise.
    #[inline]
    #[must_use]
    pub const fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Assign a source for the unique [`Id`]
    ///
    /// It must be locally unique for the current [`Ui`] hierarchy position.
    ///
    /// Ignored if [`NodesCanvas::id`] was set.
    #[inline]
    #[must_use]
    pub fn id_salt(mut self, id_salt: impl Hash) -> Self {
        self.id_salt = Id::new(id_salt);
        self
    }

    /// Set style parameters for the [`NodeGraph`] widget.
    #[inline]
    #[must_use]
    pub const fn style(mut self, style: CanvasStyle) -> Self {
        self.style = style;
        self
    }

    /// Set minimum size of the [`NodeGraph`] widget.
    #[inline]
    #[must_use]
    pub const fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }

    /// Set maximum size of the [`NodeGraph`] widget.
    #[inline]
    #[must_use]
    pub const fn max_size(mut self, max_size: Vec2) -> Self {
        self.max_size = max_size;
        self
    }

    #[inline]
    fn get_id(&self, ui_id: Id) -> Id {
        self.id.unwrap_or_else(|| ui_id.with(self.id_salt))
    }

    /// Render [`NodeGraph`] using given viewer and style into the [`Ui`].
    #[inline]
    pub fn show<T, V>(
        &self,
        node_graph: &mut NodeGraph<T>,
        viewer: &mut V,
        ui: &mut Ui,
    ) -> egui::Response
    where
        V: NodeGraphViewer<T>,
    {
        let canvas_id = self.get_id(ui.id());

        show_nodes_canvas(
            canvas_id,
            self.style,
            self.min_size,
            self.max_size,
            node_graph,
            viewer,
            ui,
        )
    }
}

#[inline(never)]
pub(super) fn show_nodes_canvas<T, V>(
    canvas_id: Id,
    mut style: CanvasStyle,
    min_size: Vec2,
    max_size: Vec2,
    node_graph: &mut NodeGraph<T>,
    viewer: &mut V,
    ui: &mut Ui,
) -> egui::Response
where
    V: NodeGraphViewer<T>,
{
    #![allow(clippy::too_many_lines)]

    let (mut latest_pos, modifiers) = ui.ctx().input(|i| (i.pointer.latest_pos(), i.modifiers));

    let bg_frame = style.get_bg_frame(ui.style());

    let outer_size_bounds = ui.available_size_before_wrap().max(min_size).min(max_size);

    let outer_resp = ui.allocate_response(outer_size_bounds, Sense::hover());

    ui.painter().add(bg_frame.paint(outer_resp.rect));

    let mut content_rect = outer_resp.rect - bg_frame.total_margin();

    // Make sure we don't shrink to the negative:
    content_rect.max.x = content_rect.max.x.max(content_rect.min.x);
    content_rect.max.y = content_rect.max.y.max(content_rect.min.y);

    let graph_layer_id = LayerId::new(ui.layer_id().order, canvas_id);

    ui.ctx().set_sublayer(ui.layer_id(), graph_layer_id);

    let mut min_scale = style.get_min_scale();
    let mut max_scale = style.get_max_scale();

    let ui_rect = content_rect;

    let mut canvas_state = CanvasState::load(
        ui.ctx(),
        canvas_id,
        node_graph,
        ui_rect,
        min_scale,
        max_scale,
    );
    let mut to_global = canvas_state.to_global();

    let clip_rect = ui.clip_rect();

    let mut ui = ui.new_child(
        UiBuilder::new()
            .ui_stack_info(UiStackInfo::new(UiKind::Frame).with_frame(bg_frame))
            .layer_id(graph_layer_id)
            .max_rect(Rect::EVERYTHING)
            .sense(Sense::click_and_drag()),
    );

    if style.get_crisp_magnified_text() {
        style.scale(max_scale);
        ui.style_mut().scale(max_scale);

        min_scale /= max_scale;
        max_scale = 1.0;
    }

    clamp_scale(&mut to_global, min_scale, max_scale, ui_rect);

    let mut panel_resp = ui.response();

    Scene::new()
        .zoom_range(min_scale..=max_scale)
        .drag_pan_buttons(DragPanButtons::SECONDARY | DragPanButtons::MIDDLE)
        .register_pan_and_zoom(&ui, &mut panel_resp, &mut to_global);

    // Inform viewer about current transform.
    viewer.current_transform(&mut to_global, node_graph);

    canvas_state.set_to_global(to_global);

    let mut from_global = to_global.inverse();

    // Full-panel drag layer (below nodes in z-order). Rect must be in **graph** (layer-local)
    // space: `ui_rect` is screen-space, but nodes use `interact` in scene space after
    // `set_transform_layer`, so a screen-space rect misaligns hit-testing with the painted panel.
    let mut viewport = (from_global * ui_rect).round_ui();

    // `Context::set_transform_layer` is sticky and affects pointer hit-testing for this layer.
    // It must run **before** `interact` so the rect we pass matches the transform used to map
    // pointer ↔ layer space (see `Scene::show` in egui, which sets transform before contents).
    ui.ctx().set_transform_layer(graph_layer_id, to_global);

    // `Ui::interact` uses `interact_rect = clip_rect().intersect(rect)` (egui `ui.rs`). The child
    // `Ui` inherits the parent's `clip_rect` in **screen** space; `viewport` is **layer-local**.
    // Intersecting those mixes coordinate systems and shrinks the drag layer to a bogus region
    // (often only one corner of the canvas). Set layer-local clip first, like `Scene::show`.
    let mut viewport_clip = from_global * clip_rect;
    ui.set_clip_rect(viewport.intersect(viewport_clip));

    // `Sense::drag()` does not participate in click hit-testing, so right-clicks never become
    // `secondary_clicked()` on this widget and `Popup::context_menu` never opens. Use
    // `click_and_drag` so empty-canvas primary/secondary clicks are attributed here (nodes still
    // win when drawn on top).
    let select_resp = ui.interact(viewport, canvas_id.with("select"), Sense::click_and_drag());
    if select_resp.dragged_by(PointerButton::Secondary)
        || select_resp.dragged_by(PointerButton::Middle)
    {
        to_global.translation += to_global.scaling * select_resp.drag_delta();
        panel_resp.mark_changed();
        canvas_state.set_to_global(to_global);
        from_global = to_global.inverse();
        viewport = (from_global * ui_rect).round_ui();
        viewport_clip = from_global * clip_rect;
        ui.set_clip_rect(viewport.intersect(viewport_clip));
        ui.ctx().set_transform_layer(graph_layer_id, to_global);
    }

    ui.expand_to_include_rect(viewport);

    // Map latest pointer position to graph space.
    latest_pos = latest_pos.map(|pos| from_global * pos);

    // Right mouse while drawing the marquee: cancel it and clear selection (press, click, or drag).
    let pointer_in_viewport = latest_pos.is_some_and(|pos| viewport.contains(pos));
    if canvas_state.is_rect_selection()
        && pointer_in_viewport
        && ui.input(|i| i.pointer.button_down(PointerButton::Secondary))
    {
        canvas_state.deselect_all_nodes();
        canvas_state.stop_rect_selection();
    }

    // Rectangle (marquee) selection: primary drag on empty canvas (nodes are drawn after this layer
    // and win hit-testing on top of nodes). Never combine with the right button held.
    let mut rect_selection_ended = None;
    if select_resp.dragged_by(PointerButton::Primary)
        && let Some(pos) = latest_pos
        && !ui.input(|i| i.pointer.button_down(PointerButton::Secondary))
    {
        if canvas_state.is_rect_selection() {
            canvas_state.update_rect_selection(pos);
        } else {
            canvas_state.start_rect_selection(pos);
        }
    }

    if select_resp.drag_stopped_by(PointerButton::Primary) {
        let min_marquee_px = ui.style().interaction.interact_radius;
        let select_rect = canvas_state.rect_selection();
        let is_tap = select_rect.is_none_or(|rect| {
            let max_edge_graph = rect.width().abs().max(rect.height().abs());
            max_edge_graph * to_global.scaling < min_marquee_px
        });

        if let Some(rect) = select_rect
            && !is_tap
        {
            rect_selection_ended = Some(rect);
        }
        if is_tap {
            canvas_state.deselect_all_nodes();
        }
        canvas_state.stop_rect_selection();
    }

    if panel_resp.changed() || select_resp.changed() {
        ui.ctx().request_repaint();
    }

    viewer.draw_background(
        style.bg_pattern.as_ref(),
        &viewport,
        &style,
        ui.style(),
        ui.painter(),
        node_graph,
    );

    let mut node_moved = None;
    let mut node_to_top = None;

    let wire_frame_size = style.get_wire_frame_size(ui.style());
    let wire_width = style.get_wire_width(ui.style());
    let wire_threshold = style.get_wire_smoothness();

    let wire_shape_idx = match style.get_wire_layer() {
        WireLayer::BehindNodes => Some(ui.painter().add(Shape::Noop)),
        WireLayer::AboveNodes => None,
    };

    let mut input_info = HashMap::new();
    let mut output_info = HashMap::new();

    let mut pin_hovered = None;

    let draw_order = canvas_state.update_draw_order(node_graph);
    let mut drag_released = false;

    let mut nodes_bb = Rect::NOTHING;
    let mut node_rects = Vec::new();

    for node_idx in draw_order {
        if !node_graph.nodes.contains(node_idx.0) {
            continue;
        }

        // show_node(node_idx);
        let response = draw_node(
            node_graph,
            &mut ui,
            node_idx,
            viewer,
            &mut canvas_state,
            &style,
            canvas_id,
            &mut input_info,
            modifiers,
            &mut output_info,
        );

        if let Some(response) = response {
            if let Some(v) = response.node_to_top {
                node_to_top = Some(v);
            }
            if let Some(v) = response.node_moved {
                node_moved = Some(v);
            }
            if let Some(v) = response.pin_hovered {
                pin_hovered = Some(v);
            }
            drag_released |= response.drag_released;

            nodes_bb = nodes_bb.union(response.final_rect);
            if rect_selection_ended.is_some() {
                node_rects.push((node_idx, response.final_rect));
            }
        }
    }

    let mut hovered_wire = None;
    let mut hovered_wire_disconnect = false;
    let mut wire_shapes = Vec::new();

    // Draw and interact with wires
    for wire in node_graph.wires.iter() {
        let Some(from_r) = output_info.get(&wire.out_pin) else {
            continue;
        };
        let Some(to_r) = input_info.get(&wire.in_pin) else {
            continue;
        };

        if !canvas_state.has_new_wires() && panel_resp.contains_pointer() && hovered_wire.is_none()
        {
            // Try to find hovered wire
            // If not dragging new wire
            // And not hovering over item above.

            if let Some(latest_pos) = latest_pos {
                let wire_hit = hit_wire(
                    ui.ctx(),
                    WireId::Connected {
                        canvas_id,
                        out_pin: wire.out_pin,
                        in_pin: wire.in_pin,
                    },
                    wire_frame_size,
                    style.get_upscale_wire_frame(),
                    style.get_downscale_wire_frame(),
                    from_r.pos,
                    to_r.pos,
                    latest_pos,
                    wire_width.max(2.0),
                    pick_wire_style(from_r.wire_style, to_r.wire_style),
                );

                if wire_hit {
                    hovered_wire = Some(wire);

                    let wire_r = ui.interact(viewport, ui.make_persistent_id(wire), Sense::click());

                    //Remove hovered wire by second click
                    hovered_wire_disconnect |= wire_r.clicked_by(PointerButton::Secondary);
                }
            }
        }

        let color = mix_colors(from_r.wire_color, to_r.wire_color);

        let mut draw_width = wire_width;
        if hovered_wire == Some(wire) {
            draw_width *= 1.5;
        }

        draw_wire(
            &ui,
            WireId::Connected {
                canvas_id,
                out_pin: wire.out_pin,
                in_pin: wire.in_pin,
            },
            &mut wire_shapes,
            wire_frame_size,
            style.get_upscale_wire_frame(),
            style.get_downscale_wire_frame(),
            from_r.pos,
            to_r.pos,
            Stroke::new(draw_width, color),
            wire_threshold,
            pick_wire_style(from_r.wire_style, to_r.wire_style),
        );
    }

    // Remove hovered wire by second click
    if hovered_wire_disconnect && let Some(wire) = hovered_wire {
        let out_pin = OutPin::new(node_graph, wire.out_pin);
        let in_pin = InPin::new(node_graph, wire.in_pin);
        viewer.disconnect(&out_pin, &in_pin, node_graph);
    }

    if let Some(select_rect) = rect_selection_ended {
        let select_nodes = node_rects.into_iter().filter_map(|(id, rect)| {
            let select = if style.get_select_rect_contained() {
                select_rect.contains_rect(rect)
            } else {
                select_rect.intersects(rect)
            };

            if select { Some(id) } else { None }
        });

        canvas_state.select_many_nodes(true, select_nodes);
    }

    if let Some(select_rect) = canvas_state.rect_selection() {
        ui.painter().rect(
            select_rect,
            0.0,
            style.get_select_fill(ui.style()),
            style.get_rect_select_stroke(),
            StrokeKind::Inside,
        );
    }

    // If right button is clicked while new wire is being dragged, cancel it.
    // This is to provide way to 'not open' the link graph node menu, but just
    // releasing the new wire to empty space.
    //
    // This uses `button_down` directly, instead of `clicked_by` to improve
    // responsiveness of the cancel action.
    if canvas_state.has_new_wires() && ui.input(|x| x.pointer.button_down(PointerButton::Secondary))
    {
        let _ = canvas_state.take_new_wires();
        panel_resp.flags.remove(Flags::CLICKED);
    }

    if style.get_centering() && select_resp.double_clicked() && nodes_bb.is_finite() {
        let nodes_bb = nodes_bb.expand(100.0);
        canvas_state.look_at(nodes_bb, ui_rect, min_scale, max_scale);
    }

    if select_resp.clicked_by(PointerButton::Primary) {
        canvas_state.deselect_all_nodes();
    }

    // Wire end position will be overridden when link graph menu is opened.
    let mut wire_end_pos = latest_pos.unwrap_or_else(|| ui_rect.center());

    if drag_released {
        let new_wires = canvas_state.take_new_wires();
        if new_wires.is_some() {
            ui.ctx().request_repaint();
        }
        match (new_wires, pin_hovered) {
            (Some(NewWires::In(in_pins)), Some(AnyPin::Out(out_pin))) => {
                for in_pin in in_pins {
                    viewer.connect(
                        &OutPin::new(node_graph, out_pin),
                        &InPin::new(node_graph, in_pin),
                        node_graph,
                    );
                }
            }
            (Some(NewWires::Out(out_pins)), Some(AnyPin::In(in_pin))) => {
                for out_pin in out_pins {
                    viewer.connect(
                        &OutPin::new(node_graph, out_pin),
                        &InPin::new(node_graph, in_pin),
                        node_graph,
                    );
                }
            }
            (Some(new_wires), None) if panel_resp.hovered() => {
                let pins = match &new_wires {
                    NewWires::In(x) => AnyPins::In(x),
                    NewWires::Out(x) => AnyPins::Out(x),
                };

                if viewer.has_dropped_wire_menu(pins, node_graph) {
                    // A wire is dropped without connecting to a pin.
                    // Show context menu for the wire drop.
                    canvas_state.set_new_wires_menu(new_wires);

                    // Force open context menu.
                    panel_resp.flags.insert(Flags::LONG_TOUCHED);
                }
            }
            _ => {}
        }
    }

    if let Some(interact_pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
        if let Some(new_wires) = canvas_state.take_new_wires_menu() {
            let pins = match &new_wires {
                NewWires::In(x) => AnyPins::In(x),
                NewWires::Out(x) => AnyPins::Out(x),
            };

            if viewer.has_dropped_wire_menu(pins, node_graph) {
                panel_resp.context_menu(|ui| {
                    let pins = match &new_wires {
                        NewWires::In(x) => AnyPins::In(x),
                        NewWires::Out(x) => AnyPins::Out(x),
                    };

                    let menu_pos = from_global * ui.cursor().min;

                    // Override wire end position when the wire-drop context menu is opened.
                    wire_end_pos = menu_pos;

                    // The context menu is opened as *link* graph menu.
                    viewer.show_dropped_wire_menu(menu_pos, ui, pins, node_graph);

                    // Even though menu could be closed in `show_dropped_wire_menu`,
                    // we need to revert the new wires here, because menu state is inaccessible.
                    // Next frame context menu won't be shown and wires will be removed.
                    canvas_state.set_new_wires_menu(new_wires);
                });
            }
        } else if viewer.has_graph_menu(interact_pos, node_graph) {
            select_resp.context_menu(|ui| {
                let menu_pos = from_global * ui.cursor().min;

                viewer.show_graph_menu(menu_pos, ui, node_graph);
            });
        }
    }

    match canvas_state.new_wires() {
        None => {}
        Some(NewWires::In(in_pins)) => {
            for &in_pin in in_pins {
                let from_pos = wire_end_pos;
                let to_r = &input_info[&in_pin];

                draw_wire(
                    &ui,
                    WireId::NewInput { canvas_id, in_pin },
                    &mut wire_shapes,
                    wire_frame_size,
                    style.get_upscale_wire_frame(),
                    style.get_downscale_wire_frame(),
                    from_pos,
                    to_r.pos,
                    Stroke::new(wire_width, to_r.wire_color),
                    wire_threshold,
                    to_r.wire_style,
                );
            }
        }
        Some(NewWires::Out(out_pins)) => {
            for &out_pin in out_pins {
                let from_r = &output_info[&out_pin];
                let to_pos = wire_end_pos;

                draw_wire(
                    &ui,
                    WireId::NewOutput { canvas_id, out_pin },
                    &mut wire_shapes,
                    wire_frame_size,
                    style.get_upscale_wire_frame(),
                    style.get_downscale_wire_frame(),
                    from_r.pos,
                    to_pos,
                    Stroke::new(wire_width, from_r.wire_color),
                    wire_threshold,
                    from_r.wire_style,
                );
            }
        }
    }

    match wire_shape_idx {
        None => {
            ui.painter().add(Shape::Vec(wire_shapes));
        }
        Some(idx) => {
            ui.painter().set(idx, Shape::Vec(wire_shapes));
        }
    }

    ui.advance_cursor_after_rect(Rect::from_min_size(ui_rect.min, Vec2::ZERO));

    if let Some(node) = node_to_top
        && node_graph.nodes.contains(node.0)
    {
        canvas_state.node_to_top(node);
    }

    if let Some((node, delta)) = node_moved
        && node_graph.nodes.contains(node.0)
    {
        ui.ctx().request_repaint();
        if canvas_state.selected_nodes().contains(&node) {
            for node in canvas_state.selected_nodes() {
                let node = &mut node_graph.nodes[node.0];
                node.pos += delta;
            }
        } else {
            let node = &mut node_graph.nodes[node.0];
            node.pos += delta;
        }
    }

    canvas_state.store(node_graph, ui.ctx());

    panel_resp
}

impl NodesCanvas {
    /// Returns list of nodes selected in the UI for the `NodesCanvas` with same id.
    ///
    /// Use same `Ui` instance that was used in [`NodesCanvas::show`].
    #[must_use]
    #[inline]
    pub fn get_selected_nodes(self, ui: &Ui) -> Vec<NodeId> {
        self.get_selected_nodes_at(ui.id(), ui.ctx())
    }

    /// Returns list of nodes selected in the UI for the `NodesCanvas` with same id.
    ///
    /// `ui_id` must be the Id of the `Ui` instance that was used in [`NodesCanvas::show`].
    #[must_use]
    #[inline]
    pub fn get_selected_nodes_at(self, ui_id: Id, ctx: &Context) -> Vec<NodeId> {
        let canvas_id = self.get_id(ui_id);
        super::graph_state::selected_nodes_for_canvas_id(canvas_id, ctx)
    }
}
