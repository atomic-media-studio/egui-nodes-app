/// 2D position in graph space (same units as Snarl node positions). No egui types.
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
