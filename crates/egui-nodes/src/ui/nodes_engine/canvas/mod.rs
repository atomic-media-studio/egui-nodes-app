//! Canvas: renders a [`NodeGraph`] inside an [`egui::Ui`] (pan/zoom, nodes, wires, selection).
//!
//! ## Coordinate spaces
//! - **Graph / scene** — node positions and marquee rectangles use the scene transform from graph
//!   state (`CanvasState::to_global`, `egui::emath::TSTransform`).
//! - **Pointer sampling** uses graph-space coordinates after mapping from screen.
//!
//! ## Input (high level)
//! - **Pan**: middle or right drag, or scroll (via [`egui::Scene::register_pan_and_zoom`]); primary does
//!   not pan. A full-panel [`egui::Sense::drag`] layer forwards secondary/middle drags so pan matches
//!   the scene transform.
//! - **Marquee select**: primary drag on empty space; nodes sit above the marquee layer and keep
//!   drags for moving/selecting. A small movement counts as a tap and clears selection instead.
//! - **Marquee cancel**: right mouse (press, click, or drag) on the canvas while drawing the marquee
//!   clears the rectangle and deselects all nodes. Marquee does not run while the right button is held.
//! - **Node drag**: primary drag on a node selects it when it was not already in the selection
//!   (multi-select is preserved when dragging a node that remains selected).
//! - **Click empty** (no drag): primary release on empty background clears selection.
//!
//! ## Submodules
//! - `style` — [`CanvasStyle`], [`NodeLayout`], selection chrome.
//! - `scene` — [`NodesCanvas`], pan/zoom, marquee, wires pass.
//! - `draw` — per-node layout and pin drawing (`draw_node`).
//! - `graph_menu` — [`apply_graph_menu_width`] and optional print helpers for [`NodeGraphViewer::show_graph_menu`].

use std::hash::Hash;

use egui::{Ui, Vec2};

use super::NodeGraph;

mod background_pattern;
mod draw;
mod graph_menu;
mod graph_state;
mod node_viewer;
mod pin;
mod scene;
mod style;
mod transform;
mod wire;

pub use self::{
    background_pattern::{BackgroundPattern, Grid, GridRenderMode},
    graph_menu::{
        apply_graph_menu_width, print_graph_menu_button_clicked, print_graph_menu_float_clicked,
        print_graph_menu_int_clicked, print_graph_menu_sink_clicked, print_graph_menu_string_clicked,
        GRAPH_MENU_DEFAULT_WIDTH,
    },
    graph_state::get_selected_nodes,
    node_viewer::NodeGraphViewer,
    pin::{AnyPins, GraphPin, PinInfo, PinShape, PinWireInfo},
    scene::NodesCanvas,
    wire::{WireLayer, WireStyle},
};

pub use style::{CanvasStyle, NodeLayout, NodeLayoutKind, PinPlacement, SelectionStyle};

pub(crate) use style::{
    default_rect_selection_stroke, default_selection_fill, default_selection_stroke,
};

impl<T> NodeGraph<T> {
    /// Render [`NodeGraph`] using given viewer and style into the [`Ui`].
    #[inline]
    pub fn show<V>(
        &mut self,
        viewer: &mut V,
        style: &style::CanvasStyle,
        id_salt: impl Hash,
        ui: &mut Ui,
    ) where
        V: NodeGraphViewer<T>,
    {
        scene::show_nodes_canvas(
            ui.make_persistent_id(id_salt),
            *style,
            Vec2::ZERO,
            Vec2::INFINITY,
            self,
            viewer,
            ui,
        );
    }
}

#[test]
const fn canvas_style_is_send_sync() {
    const fn is_send_sync<T: Send + Sync>() {}
    is_send_sync::<style::CanvasStyle>();
}
