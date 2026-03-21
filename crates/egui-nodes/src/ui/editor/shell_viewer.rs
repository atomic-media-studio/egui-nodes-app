//! [`NodesShellViewer`] decorates a [`NodeGraphViewer`] with [`NodesStyle`] strokes and
//! [`InteractionMode`] (e.g. inspect).

use std::sync::Arc;

use egui::emath::TSTransform;
use egui::{Context, Frame, Id, Painter, Pos2, Rect, Style, Ui};

use crate::ui::nodes_engine::{InPin, InPinId, NodeId, OutPin, OutPinId, NodeGraph};
use crate::ui::nodes_canvas::{
    AnyPins, BackgroundPattern, CanvasStyle, NodeGraphViewer, get_selected_nodes,
};

use crate::ui::state::InteractionMode;
use crate::ui::style::NodesStyle;

/// Wraps your [`NodeGraphViewer`] to apply [`NodesStyle`] node strokes and enforce [`InteractionMode::Inspect`].
pub struct NodesShellViewer<V> {
    pub inner: V,
    style: Arc<NodesStyle>,
    canvas_id: Id,
    mode: InteractionMode,
    ctx: Option<Context>,
}

impl<V> NodesShellViewer<V> {
    pub fn new(inner: V) -> Self {
        Self {
            inner,
            style: Arc::new(NodesStyle::new()),
            canvas_id: Id::NULL,
            mode: InteractionMode::Select,
            ctx: None,
        }
    }

    pub(crate) fn prepare(
        &mut self,
        canvas_id: Id,
        mode: InteractionMode,
        ctx: &Context,
        style: &NodesStyle,
    ) {
        self.canvas_id = canvas_id;
        self.mode = mode;
        self.ctx = Some(ctx.clone());
        self.style = Arc::new(style.clone());
    }

    fn inspect(&self) -> bool {
        self.mode == InteractionMode::Inspect
    }
}

impl<T, V: NodeGraphViewer<T>> NodeGraphViewer<T> for NodesShellViewer<V> {
    fn title(&mut self, node: &T) -> String {
        self.inner.title(node)
    }

    fn node_frame(
        &mut self,
        default: Frame,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        node_graph: &NodeGraph<T>,
    ) -> Frame {
        let mut frame = self
            .inner
            .node_frame(default, node, inputs, outputs, node_graph);
        let Some(ref ctx) = self.ctx else {
            return frame;
        };
        let selected = get_selected_nodes(self.canvas_id, ctx).contains(&node);
        let egui_style = ctx.style();
        frame.stroke = self.style.node_style.stroke(
            selected,
            false,
            None,
            frame.stroke,
            egui_style.as_ref(),
        );
        frame
    }

    fn header_frame(
        &mut self,
        default: Frame,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        node_graph: &NodeGraph<T>,
    ) -> Frame {
        self.inner
            .header_frame(default, node, inputs, outputs, node_graph)
    }

    fn has_node_style(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        node_graph: &NodeGraph<T>,
    ) -> bool {
        self.inner.has_node_style(node, inputs, outputs, node_graph)
    }

    fn apply_node_style(
        &mut self,
        style: &mut Style,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        node_graph: &NodeGraph<T>,
    ) {
        self.inner
            .apply_node_style(style, node, inputs, outputs, node_graph)
    }

    fn node_layout(
        &mut self,
        default: crate::ui::nodes_canvas::NodeLayout,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        node_graph: &NodeGraph<T>,
    ) -> crate::ui::nodes_canvas::NodeLayout {
        self.inner
            .node_layout(default, node, inputs, outputs, node_graph)
    }

    fn show_header(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        node_graph: &mut NodeGraph<T>,
    ) {
        self.inner.show_header(node, inputs, outputs, ui, node_graph)
    }

    fn inputs(&mut self, node: &T) -> usize {
        self.inner.inputs(node)
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        node_graph: &mut NodeGraph<T>,
    ) -> crate::ui::nodes_canvas::PinInfo {
        self.inner.show_input(pin, ui, node_graph)
    }

    fn outputs(&mut self, node: &T) -> usize {
        self.inner.outputs(node)
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut Ui,
        node_graph: &mut NodeGraph<T>,
    ) -> crate::ui::nodes_canvas::PinInfo {
        self.inner.show_output(pin, ui, node_graph)
    }

    fn has_body(&mut self, node: &T) -> bool {
        self.inner.has_body(node)
    }

    fn show_body(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        node_graph: &mut NodeGraph<T>,
    ) {
        self.inner.show_body(node, inputs, outputs, ui, node_graph)
    }

    fn has_footer(&mut self, node: &T) -> bool {
        self.inner.has_footer(node)
    }

    fn show_footer(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        node_graph: &mut NodeGraph<T>,
    ) {
        self.inner.show_footer(node, inputs, outputs, ui, node_graph)
    }

    fn final_node_rect(
        &mut self,
        node: NodeId,
        rect: Rect,
        ui: &mut Ui,
        node_graph: &mut NodeGraph<T>,
    ) {
        self.inner.final_node_rect(node, rect, ui, node_graph)
    }

    fn has_on_hover_popup(&mut self, node: &T) -> bool {
        self.inner.has_on_hover_popup(node)
    }

    fn show_on_hover_popup(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        node_graph: &mut NodeGraph<T>,
    ) {
        self.inner
            .show_on_hover_popup(node, inputs, outputs, ui, node_graph)
    }

    fn has_wire_widget(&mut self, from: &OutPinId, to: &InPinId, node_graph: &NodeGraph<T>) -> bool {
        self.inner.has_wire_widget(from, to, node_graph)
    }

    fn show_wire_widget(
        &mut self,
        from: &OutPin,
        to: &InPin,
        ui: &mut Ui,
        node_graph: &mut NodeGraph<T>,
    ) {
        self.inner.show_wire_widget(from, to, ui, node_graph)
    }

    fn has_graph_menu(&mut self, pos: Pos2, node_graph: &mut NodeGraph<T>) -> bool {
        if self.inspect() {
            return false;
        }
        self.inner.has_graph_menu(pos, node_graph)
    }

    fn show_graph_menu(&mut self, pos: Pos2, ui: &mut Ui, node_graph: &mut NodeGraph<T>) {
        if self.inspect() {
            return;
        }
        self.inner.show_graph_menu(pos, ui, node_graph)
    }

    fn has_dropped_wire_menu(&mut self, src_pins: AnyPins, node_graph: &mut NodeGraph<T>) -> bool {
        if self.inspect() {
            return false;
        }
        self.inner.has_dropped_wire_menu(src_pins, node_graph)
    }

    fn show_dropped_wire_menu(
        &mut self,
        pos: Pos2,
        ui: &mut Ui,
        src_pins: AnyPins,
        node_graph: &mut NodeGraph<T>,
    ) {
        if self.inspect() {
            return;
        }
        self.inner.show_dropped_wire_menu(pos, ui, src_pins, node_graph)
    }

    fn has_node_menu(&mut self, node: &T) -> bool {
        if self.inspect() {
            return false;
        }
        self.inner.has_node_menu(node)
    }

    fn show_node_menu(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        node_graph: &mut NodeGraph<T>,
    ) {
        if self.inspect() {
            return;
        }
        self.inner
            .show_node_menu(node, inputs, outputs, ui, node_graph)
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, node_graph: &mut NodeGraph<T>) {
        if self.inspect() {
            return;
        }
        self.inner.connect(from, to, node_graph)
    }

    fn disconnect(&mut self, from: &OutPin, to: &InPin, node_graph: &mut NodeGraph<T>) {
        if self.inspect() {
            return;
        }
        self.inner.disconnect(from, to, node_graph)
    }

    fn drop_outputs(&mut self, pin: &OutPin, node_graph: &mut NodeGraph<T>) {
        if self.inspect() {
            return;
        }
        self.inner.drop_outputs(pin, node_graph)
    }

    fn drop_inputs(&mut self, pin: &InPin, node_graph: &mut NodeGraph<T>) {
        if self.inspect() {
            return;
        }
        self.inner.drop_inputs(pin, node_graph)
    }

    fn draw_background(
        &mut self,
        background: Option<&BackgroundPattern>,
        viewport: &Rect,
        canvas_style: &CanvasStyle,
        style: &Style,
        painter: &Painter,
        node_graph: &NodeGraph<T>,
    ) {
        self.inner
            .draw_background(background, viewport, canvas_style, style, painter, node_graph)
    }

    fn current_transform(&mut self, to_global: &mut TSTransform, node_graph: &mut NodeGraph<T>) {
        self.inner.current_transform(to_global, node_graph)
    }
}
