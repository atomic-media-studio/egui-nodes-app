use egui::{emath::TSTransform, Painter, Pos2, Rect, Style, Ui};

use super::super::{InPin, InPinId, NodeId, OutPin, OutPinId, NodeGraph};

use super::{
    pin::{AnyPins, PinInfo},
    BackgroundPattern, NodeLayout, CanvasStyle,
};

/// Renders and handles interaction for a [`NodeGraph`](crate::ui::nodes_engine::NodeGraph).
///
/// Implementations supply node chrome (title, pins, body) and optional menus / wire behavior.
pub trait NodeGraphViewer<T> {
    /// Returns title of the node.
    fn title(&mut self, node: &T) -> String;

    /// Returns the node's frame.
    /// All node's elements will be rendered inside this frame.
    /// Except for pins if they are configured to be rendered outside of the frame.
    ///
    /// Returns `default` by default.
    /// `default` frame is taken from the [`CanvasStyle::node_frame`] or constructed if it's `None`.
    ///
    /// Override this method to customize the frame for specific nodes.
    fn node_frame(
        &mut self,
        default: egui::Frame,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        node_graph: &NodeGraph<T>,
    ) -> egui::Frame {
        let _ = (node, inputs, outputs, node_graph);
        default
    }

    /// Returns the node's header frame.
    ///
    /// This frame would be placed on top of the node's frame.
    /// And header UI (see [`show_header`]) will be placed inside this frame.
    ///
    /// Returns `default` by default.
    /// `default` frame is taken from the [`CanvasStyle::header_frame`],
    /// or [`CanvasStyle::node_frame`] with removed shadow if `None`,
    /// or constructed if both are `None`.
    fn header_frame(
        &mut self,
        default: egui::Frame,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        node_graph: &NodeGraph<T>,
    ) -> egui::Frame {
        let _ = (node, inputs, outputs, node_graph);
        default
    }
    /// Checks if node has a custom egui style.
    #[inline]
    fn has_node_style(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        node_graph: &NodeGraph<T>,
    ) -> bool {
        let _ = (node, inputs, outputs, node_graph);
        false
    }

    /// Modifies the node's egui style
    fn apply_node_style(
        &mut self,
        style: &mut Style,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        node_graph: &NodeGraph<T>,
    ) {
        let _ = (style, node, inputs, outputs, node_graph);
    }

    /// Returns elements layout for the node.
    ///
    /// Node consists of 5 parts: header, body, footer, input pins and output pins.
    /// See [`NodeLayout`] for available placements.
    ///
    /// Returns `default` by default.
    /// `default` layout is taken from the [`CanvasStyle::node_layout`] or constructed if it's `None`.
    /// Override this method to customize the layout for specific nodes.
    #[inline]
    fn node_layout(
        &mut self,
        default: NodeLayout,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        node_graph: &NodeGraph<T>,
    ) -> NodeLayout {
        let _ = (node, inputs, outputs, node_graph);
        default
    }

    /// Renders elements inside the node's header frame.
    ///
    /// This is the good place to show the node's title and controls related to the whole node.
    ///
    /// By default it shows the node's title.
    #[inline]
    fn show_header(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        node_graph: &mut NodeGraph<T>,
    ) {
        let _ = (inputs, outputs);
        ui.label(self.title(&node_graph[node]));
    }

    /// Returns number of input pins of the node.
    ///
    /// [`NodeGraphViewer::show_input`] will be called for each input in range `0..inputs()`.
    fn inputs(&mut self, node: &T) -> usize;

    /// Renders one specified node's input element and returns drawer for the corresponding pin.
    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        node_graph: &mut NodeGraph<T>,
    ) -> PinInfo;

    /// Returns number of output pins of the node.
    ///
    /// [`NodeGraphViewer::show_output`] will be called for each output in range `0..outputs()`.
    fn outputs(&mut self, node: &T) -> usize;

    /// Renders the node's output.
    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut Ui,
        node_graph: &mut NodeGraph<T>,
    ) -> PinInfo;

    /// Checks if node has something to show in body - between input and output pins.
    #[inline]
    fn has_body(&mut self, node: &T) -> bool {
        let _ = node;
        false
    }

    /// Renders the node's body.
    #[inline]
    fn show_body(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        node_graph: &mut NodeGraph<T>,
    ) {
        let _ = (node, inputs, outputs, ui, node_graph);
    }

    /// Checks if node has something to show in footer - below pins and body.
    #[inline]
    fn has_footer(&mut self, node: &T) -> bool {
        let _ = node;
        false
    }

    /// Renders the node's footer.
    #[inline]
    fn show_footer(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        node_graph: &mut NodeGraph<T>,
    ) {
        let _ = (node, inputs, outputs, ui, node_graph);
    }

    /// Reports the final node's rect after rendering.
    ///
    /// It aimed to be used for custom positioning of nodes that requires node dimensions for calculations.
    /// Node's position can be modified directly in this method.
    #[inline]
    fn final_node_rect(&mut self, node: NodeId, rect: Rect, ui: &mut Ui, node_graph: &mut NodeGraph<T>) {
        let _ = (node, rect, ui, node_graph);
    }

    /// Checks if node has something to show in on-hover popup.
    #[inline]
    fn has_on_hover_popup(&mut self, node: &T) -> bool {
        let _ = node;
        false
    }

    /// Renders the node's on-hover popup.
    #[inline]
    fn show_on_hover_popup(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        node_graph: &mut NodeGraph<T>,
    ) {
        let _ = (node, inputs, outputs, ui, node_graph);
    }

    /// Checks if wire has something to show in widget.
    /// This may not be called if wire is invisible.
    #[inline]
    fn has_wire_widget(&mut self, from: &OutPinId, to: &InPinId, node_graph: &NodeGraph<T>) -> bool {
        let _ = (from, to, node_graph);
        false
    }

    /// Renders the wire's widget.
    /// This may not be called if wire is invisible.
    #[inline]
    fn show_wire_widget(&mut self, from: &OutPin, to: &InPin, ui: &mut Ui, node_graph: &mut NodeGraph<T>) {
        let _ = (from, to, ui, node_graph);
    }

    /// Whether to show a context menu on empty space at `pos` (right-click or long-press).
    #[inline]
    fn has_graph_menu(&mut self, pos: Pos2, node_graph: &mut NodeGraph<T>) -> bool {
        let _ = (pos, node_graph);
        false
    }

    /// Context menu for empty space (e.g. “add node”).
    #[inline]
    fn show_graph_menu(&mut self, pos: Pos2, ui: &mut Ui, node_graph: &mut NodeGraph<T>) {
        let _ = (pos, ui, node_graph);
    }

    /// Whether to show a menu when a wire drag ends without a valid target.
    #[inline]
    fn has_dropped_wire_menu(&mut self, src_pins: AnyPins, node_graph: &mut NodeGraph<T>) -> bool {
        let _ = (src_pins, node_graph);
        false
    }

    /// Menu when a pin is released on empty space (e.g. spawn a node and connect the wire).
    #[inline]
    fn show_dropped_wire_menu(
        &mut self,
        pos: Pos2,
        ui: &mut Ui,
        src_pins: AnyPins,
        node_graph: &mut NodeGraph<T>,
    ) {
        let _ = (pos, ui, src_pins, node_graph);
    }

    /// Checks if the node has something to show in context menu if right-clicked or long-touched on the node.
    #[inline]
    fn has_node_menu(&mut self, node: &T) -> bool {
        let _ = node;
        false
    }

    /// Context menu for a node (right-click on the node).
    #[inline]
    fn show_node_menu(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        node_graph: &mut NodeGraph<T>,
    ) {
        let _ = (node, inputs, outputs, ui, node_graph);
    }

    /// Asks the viewer to connect two pins.
    ///
    /// Called when the user finishes dragging a wire between pins (default: connect in the graph).
    #[inline]
    fn connect(&mut self, from: &OutPin, to: &InPin, node_graph: &mut NodeGraph<T>) {
        node_graph.connect(from.id, to.id);
    }

    /// Asks the viewer to disconnect two pins.
    #[inline]
    fn disconnect(&mut self, from: &OutPin, to: &InPin, node_graph: &mut NodeGraph<T>) {
        node_graph.disconnect(from.id, to.id);
    }

    /// Asks the viewer to disconnect all wires from the output pin.
    ///
    /// Called when the user clears wires from an output pin (default: drop all outputs).
    #[inline]
    fn drop_outputs(&mut self, pin: &OutPin, node_graph: &mut NodeGraph<T>) {
        node_graph.drop_outputs(pin.id);
    }

    /// Asks the viewer to disconnect all wires from the input pin.
    ///
    /// Called when the user clears wires from an input pin (default: drop all inputs).
    #[inline]
    fn drop_inputs(&mut self, pin: &InPin, node_graph: &mut NodeGraph<T>) {
        node_graph.drop_inputs(pin.id);
    }

    /// Draws the canvas background behind nodes and wires.
    ///
    /// By default it draws the background pattern using [`BackgroundPattern::draw`].
    ///
    /// If you want to draw the background yourself, you can override this method.
    #[inline]
    fn draw_background(
        &mut self,
        background: Option<&BackgroundPattern>,
        viewport: &Rect,
        canvas_style: &CanvasStyle,
        style: &Style,
        painter: &Painter,
        node_graph: &NodeGraph<T>,
    ) {
        let _ = node_graph;

        if let Some(background) = background {
            background.draw(viewport, canvas_style, style, painter);
        }
    }

    /// Scene transform before nodes are drawn; override to adjust pan/zoom (e.g. initial scale).
    ///
    /// By default it does nothing.
    #[inline]
    fn current_transform(&mut self, to_global: &mut TSTransform, node_graph: &mut NodeGraph<T>) {
        let _ = (to_global, node_graph);
    }
}
