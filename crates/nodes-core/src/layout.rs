//! 2D layout without depending on egui — convert in `nodes-snarl` to [`egui::Pos2`].

/// Node position in graph space (same units as egui Snarl node positions).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Layout2d {
    pub x: f32,
    pub y: f32,
}

impl Layout2d {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
