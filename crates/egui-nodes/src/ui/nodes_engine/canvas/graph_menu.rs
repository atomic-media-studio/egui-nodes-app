//! Helpers for [`NodeGraphViewer::show_graph_menu`](super::NodeGraphViewer::show_graph_menu):
//! default popup width and optional [`println!`] hooks for debugging or telemetry.
//!
//! Call [`apply_graph_menu_width`] at the start of your menu UI so the panel matches library defaults.

use egui::Ui;

/// Default width (pixels) for the empty-canvas graph context menu (`NodesCanvas` / scene pass).
pub const GRAPH_MENU_DEFAULT_WIDTH: f32 = 160.0;

/// Sets the graph context menu to [`GRAPH_MENU_DEFAULT_WIDTH`]. Call at the top of [`NodeGraphViewer::show_graph_menu`](super::NodeGraphViewer::show_graph_menu).
#[inline]
pub fn apply_graph_menu_width(ui: &mut Ui) {
    ui.set_min_width(GRAPH_MENU_DEFAULT_WIDTH);
    ui.set_max_width(GRAPH_MENU_DEFAULT_WIDTH);
}

/// Prints `graph menu: Button` to stdout (e.g. demo / debugging).
#[inline]
pub fn print_graph_menu_button_clicked() {
    println!("graph menu: Button");
}

/// Prints `graph menu: Int` to stdout.
#[inline]
pub fn print_graph_menu_int_clicked() {
    println!("graph menu: Int");
}

/// Prints `graph menu: String` to stdout.
#[inline]
pub fn print_graph_menu_string_clicked() {
    println!("graph menu: String");
}

/// Prints `graph menu: Float` to stdout.
#[inline]
pub fn print_graph_menu_float_clicked() {
    println!("graph menu: Float");
}

/// Prints `graph menu: Sink` to stdout.
#[inline]
pub fn print_graph_menu_sink_clicked() {
    println!("graph menu: Sink");
}
