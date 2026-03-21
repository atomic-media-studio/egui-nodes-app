use egui::{
    emath::Rot2,
    vec2,
    Color32,
    Painter,
    Rect,
    Stroke,
    Style,
    Vec2,
};

use super::SnarlStyle;

/// How a [`Grid`] is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "egui-probe", derive(egui_probe::EguiProbe))]
pub enum GridRenderMode {
    /// Full vertical + horizontal line segments.
    #[default]
    Lines,
    /// Filled circles only at grid line intersections (no line segments).
    Dots,
}

/// Grid background pattern.
/// Stroke defaults come from [`SnarlStyle::bg_pattern_stroke`]; use [`Grid::color`] to override color
/// per pattern.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "egui-probe", derive(egui_probe::EguiProbe))]
pub struct Grid {
    /// Spacing between grid lines / dot rows.
    pub spacing: Vec2,

    /// Angle of the grid (radians).
    #[cfg_attr(feature = "egui-probe", egui_probe(as egui_probe::angle))]
    pub angle: f32,

    /// Draw lines, dots at intersections, etc.
    pub mode: GridRenderMode,

    /// Offset in pattern space before rotation (animate for drift / parallax).
    pub phase: Vec2,

    /// Radius for [`GridRenderMode::Dots`] (logical pixels; scaled when the Snarl zoom changes).
    pub dot_radius: f32,

    /// Optional color override. If `None`, uses [`SnarlStyle::get_bg_pattern_stroke`] color.
    pub color: Option<Color32>,
}

const DEFAULT_GRID_SPACING: Vec2 = vec2(50.0, 50.0);
macro_rules! default_grid_spacing {
    () => {
        stringify!(vec2(50.0, 50.0))
    };
}

const DEFAULT_GRID_ANGLE: f32 = 1.0;
macro_rules! default_grid_angle {
    () => {
        stringify!(1.0)
    };
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            spacing: DEFAULT_GRID_SPACING,
            angle: DEFAULT_GRID_ANGLE,
            mode: GridRenderMode::Lines,
            phase: Vec2::ZERO,
            dot_radius: 2.0,
            color: None,
        }
    }
}

impl Grid {
    /// Create new grid with given spacing and angle (line mode, no phase, default dot radius).
    #[must_use]
    pub const fn new(spacing: Vec2, angle: f32) -> Self {
        Self {
            spacing,
            angle,
            mode: GridRenderMode::Lines,
            phase: Vec2::ZERO,
            dot_radius: 2.0,
            color: None,
        }
    }

    /// Grid with [`GridRenderMode::Dots`] at intersections.
    #[must_use]
    pub fn dots(spacing: Vec2, angle: f32, dot_radius: f32) -> Self {
        Self {
            spacing,
            angle,
            mode: GridRenderMode::Dots,
            phase: Vec2::ZERO,
            dot_radius,
            color: None,
        }
    }

    fn draw(&self, viewport: &Rect, snarl_style: &SnarlStyle, style: &Style, painter: &Painter) {
        let bg_stroke = snarl_style.get_bg_pattern_stroke(style);
        let color = self.color.unwrap_or(bg_stroke.color);
        let stroke = Stroke::new(bg_stroke.width, color);

        let spacing = vec2(self.spacing.x.max(1.0), self.spacing.y.max(1.0));

        let rot = Rot2::from_angle(self.angle);
        let rot_inv = rot.inverse();

        let pattern_bounds = viewport.rotate_bb(rot_inv);

        let min_x = (pattern_bounds.min.x / spacing.x).ceil();
        let max_x = (pattern_bounds.max.x / spacing.x).floor();
        let min_y = (pattern_bounds.min.y / spacing.y).ceil();
        let max_y = (pattern_bounds.max.y / spacing.y).floor();

        match self.mode {
            GridRenderMode::Lines => {
                Self::draw_lines(
                    pattern_bounds,
                    spacing,
                    rot,
                    min_x,
                    max_x,
                    min_y,
                    max_y,
                    stroke,
                    self.phase,
                    painter,
                );
            }
            GridRenderMode::Dots => {
                Self::draw_dots(
                    spacing,
                    rot,
                    min_x,
                    max_x,
                    min_y,
                    max_y,
                    self.phase,
                    self.dot_radius.max(0.25),
                    color,
                    painter,
                );
            }
        }
    }

    fn draw_lines(
        pattern_bounds: Rect,
        spacing: Vec2,
        rot: Rot2,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
        stroke: Stroke,
        phase: Vec2,
        painter: &Painter,
    ) {
        #[allow(clippy::cast_possible_truncation)]
        for x in 0..=f32::ceil(max_x - min_x) as i64 {
            #[allow(clippy::cast_precision_loss)]
            let x = (x as f32 + min_x) * spacing.x;

            let top = (rot * (vec2(x, pattern_bounds.min.y) + phase)).to_pos2();
            let bottom = (rot * (vec2(x, pattern_bounds.max.y) + phase)).to_pos2();

            painter.line_segment([top, bottom], stroke);
        }

        #[allow(clippy::cast_possible_truncation)]
        for y in 0..=f32::ceil(max_y - min_y) as i64 {
            #[allow(clippy::cast_precision_loss)]
            let y = (y as f32 + min_y) * spacing.y;

            let top = (rot * (vec2(pattern_bounds.min.x, y) + phase)).to_pos2();
            let bottom = (rot * (vec2(pattern_bounds.max.x, y) + phase)).to_pos2();

            painter.line_segment([top, bottom], stroke);
        }
    }

    fn draw_dots(
        spacing: Vec2,
        rot: Rot2,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
        phase: Vec2,
        dot_radius: f32,
        color: Color32,
        painter: &Painter,
    ) {
        #[allow(clippy::cast_possible_truncation)]
        let nx = f32::ceil(max_x - min_x) as i64;
        #[allow(clippy::cast_possible_truncation)]
        let ny = f32::ceil(max_y - min_y) as i64;

        for ix in 0..=nx {
            #[allow(clippy::cast_precision_loss)]
            let x = (ix as f32 + min_x) * spacing.x;
            for iy in 0..=ny {
                #[allow(clippy::cast_precision_loss)]
                let y = (iy as f32 + min_y) * spacing.y;
                let p = (rot * (vec2(x, y) + phase)).to_pos2();
                painter.circle_filled(p, dot_radius, color);
            }
        }
    }
}

/// Background pattern show beneath nodes and wires.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "egui-probe", derive(egui_probe::EguiProbe))]
pub enum BackgroundPattern {
    /// No pattern.
    NoPattern,

    /// Linear grid (lines and/or dots).
    #[cfg_attr(feature = "egui-probe", egui_probe(transparent))]
    Grid(Grid),
}

impl Default for BackgroundPattern {
    fn default() -> Self {
        BackgroundPattern::new()
    }
}

impl BackgroundPattern {
    /// Create new background pattern with default values.
    ///
    /// Default patter is `Grid` with spacing - `
    #[doc = default_grid_spacing!()]
    /// ` and angle - `
    #[doc = default_grid_angle!()]
    /// ` radian.
    #[must_use]
    pub const fn new() -> Self {
        Self::Grid(Grid::new(DEFAULT_GRID_SPACING, DEFAULT_GRID_ANGLE))
    }

    /// Create new grid background pattern with given spacing and angle.
    #[must_use]
    pub const fn grid(spacing: Vec2, angle: f32) -> Self {
        Self::Grid(Grid::new(spacing, angle))
    }

    /// Draws background pattern.
    pub fn draw(
        &self,
        viewport: &Rect,
        snarl_style: &SnarlStyle,
        style: &Style,
        painter: &Painter,
    ) {
        match self {
            BackgroundPattern::Grid(g) => g.draw(viewport, snarl_style, style, painter),
            BackgroundPattern::NoPattern => {}
        }
    }
}
