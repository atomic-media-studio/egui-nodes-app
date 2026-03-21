//! Map between [`nodes_core::Layout2d`] and egui positions.

use egui::Pos2;

use nodes_core::Layout2d;

#[inline]
pub fn layout_to_pos2(layout: Layout2d) -> Pos2 {
    Pos2::new(layout.x, layout.y)
}

#[inline]
pub fn pos2_to_layout(pos: Pos2) -> Layout2d {
    Layout2d::new(pos.x, pos.y)
}
