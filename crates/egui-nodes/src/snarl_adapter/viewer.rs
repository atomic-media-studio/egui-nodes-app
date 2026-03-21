use std::sync::Arc;

use egui::emath::TSTransform;
use egui::{Context, Frame, Id, Painter, Pos2, Rect, Style, Ui};

use egui_snarl_fork::ui::{
    AnyPins, BackgroundPattern, SnarlStyle, SnarlViewer, get_selected_nodes,
};
use egui_snarl_fork::{InPin, InPinId, NodeId, OutPin, OutPinId, Snarl};

use crate::ui::state::InteractionMode;
use crate::ui::style::NodesStyle;

/// Wraps your [`SnarlViewer`] to apply [`NodesStyle`] node strokes and enforce [`InteractionMode::Inspect`].
pub struct NodesShellViewer<V> {
    pub inner: V,
    style: Arc<NodesStyle>,
    snarl_id: Id,
    mode: InteractionMode,
    ctx: Option<Context>,
}

impl<V> NodesShellViewer<V> {
    pub fn new(inner: V) -> Self {
        Self {
            inner,
            style: Arc::new(NodesStyle::new()),
            snarl_id: Id::NULL,
            mode: InteractionMode::Select,
            ctx: None,
        }
    }

    pub(crate) fn prepare(
        &mut self,
        snarl_id: Id,
        mode: InteractionMode,
        ctx: &Context,
        style: &NodesStyle,
    ) {
        self.snarl_id = snarl_id;
        self.mode = mode;
        self.ctx = Some(ctx.clone());
        self.style = Arc::new(style.clone());
    }

    fn inspect(&self) -> bool {
        self.mode == InteractionMode::Inspect
    }
}

impl<T, V: SnarlViewer<T>> SnarlViewer<T> for NodesShellViewer<V> {
    fn title(&mut self, node: &T) -> String {
        self.inner.title(node)
    }

    fn node_frame(
        &mut self,
        default: Frame,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        snarl: &Snarl<T>,
    ) -> Frame {
        let mut frame = self
            .inner
            .node_frame(default, node, inputs, outputs, snarl);
        let Some(ref ctx) = self.ctx else {
            return frame;
        };
        let selected = get_selected_nodes(self.snarl_id, ctx).contains(&node);
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
        snarl: &Snarl<T>,
    ) -> Frame {
        self.inner
            .header_frame(default, node, inputs, outputs, snarl)
    }

    fn has_node_style(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        snarl: &Snarl<T>,
    ) -> bool {
        self.inner.has_node_style(node, inputs, outputs, snarl)
    }

    fn apply_node_style(
        &mut self,
        style: &mut Style,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        snarl: &Snarl<T>,
    ) {
        self.inner
            .apply_node_style(style, node, inputs, outputs, snarl)
    }

    fn node_layout(
        &mut self,
        default: egui_snarl_fork::ui::NodeLayout,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        snarl: &Snarl<T>,
    ) -> egui_snarl_fork::ui::NodeLayout {
        self.inner
            .node_layout(default, node, inputs, outputs, snarl)
    }

    fn show_header(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<T>,
    ) {
        self.inner.show_header(node, inputs, outputs, ui, snarl)
    }

    fn inputs(&mut self, node: &T) -> usize {
        self.inner.inputs(node)
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<T>,
    ) -> impl egui_snarl_fork::ui::SnarlPin + 'static {
        self.inner.show_input(pin, ui, snarl)
    }

    fn outputs(&mut self, node: &T) -> usize {
        self.inner.outputs(node)
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<T>,
    ) -> impl egui_snarl_fork::ui::SnarlPin + 'static {
        self.inner.show_output(pin, ui, snarl)
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
        snarl: &mut Snarl<T>,
    ) {
        self.inner.show_body(node, inputs, outputs, ui, snarl)
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
        snarl: &mut Snarl<T>,
    ) {
        self.inner.show_footer(node, inputs, outputs, ui, snarl)
    }

    fn final_node_rect(
        &mut self,
        node: NodeId,
        rect: Rect,
        ui: &mut Ui,
        snarl: &mut Snarl<T>,
    ) {
        self.inner.final_node_rect(node, rect, ui, snarl)
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
        snarl: &mut Snarl<T>,
    ) {
        self.inner
            .show_on_hover_popup(node, inputs, outputs, ui, snarl)
    }

    fn has_wire_widget(&mut self, from: &OutPinId, to: &InPinId, snarl: &Snarl<T>) -> bool {
        self.inner.has_wire_widget(from, to, snarl)
    }

    fn show_wire_widget(
        &mut self,
        from: &OutPin,
        to: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<T>,
    ) {
        self.inner.show_wire_widget(from, to, ui, snarl)
    }

    fn has_graph_menu(&mut self, pos: Pos2, snarl: &mut Snarl<T>) -> bool {
        if self.inspect() {
            return false;
        }
        self.inner.has_graph_menu(pos, snarl)
    }

    fn show_graph_menu(&mut self, pos: Pos2, ui: &mut Ui, snarl: &mut Snarl<T>) {
        if self.inspect() {
            return;
        }
        self.inner.show_graph_menu(pos, ui, snarl)
    }

    fn has_dropped_wire_menu(&mut self, src_pins: AnyPins, snarl: &mut Snarl<T>) -> bool {
        if self.inspect() {
            return false;
        }
        self.inner.has_dropped_wire_menu(src_pins, snarl)
    }

    fn show_dropped_wire_menu(
        &mut self,
        pos: Pos2,
        ui: &mut Ui,
        src_pins: AnyPins,
        snarl: &mut Snarl<T>,
    ) {
        if self.inspect() {
            return;
        }
        self.inner.show_dropped_wire_menu(pos, ui, src_pins, snarl)
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
        snarl: &mut Snarl<T>,
    ) {
        if self.inspect() {
            return;
        }
        self.inner
            .show_node_menu(node, inputs, outputs, ui, snarl)
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<T>) {
        if self.inspect() {
            return;
        }
        self.inner.connect(from, to, snarl)
    }

    fn disconnect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<T>) {
        if self.inspect() {
            return;
        }
        self.inner.disconnect(from, to, snarl)
    }

    fn drop_outputs(&mut self, pin: &OutPin, snarl: &mut Snarl<T>) {
        if self.inspect() {
            return;
        }
        self.inner.drop_outputs(pin, snarl)
    }

    fn drop_inputs(&mut self, pin: &InPin, snarl: &mut Snarl<T>) {
        if self.inspect() {
            return;
        }
        self.inner.drop_inputs(pin, snarl)
    }

    fn draw_background(
        &mut self,
        background: Option<&BackgroundPattern>,
        viewport: &Rect,
        snarl_style: &SnarlStyle,
        style: &Style,
        painter: &Painter,
        snarl: &Snarl<T>,
    ) {
        self.inner
            .draw_background(background, viewport, snarl_style, style, painter, snarl)
    }

    fn current_transform(&mut self, to_global: &mut TSTransform, snarl: &mut Snarl<T>) {
        self.inner.current_transform(to_global, snarl)
    }
}
