//! Node layout and [`CanvasStyle`] — pin placement, chrome, wires, background, zoom, and selection.
//!
//! Split from interaction/rendering so callers can serialize and tune appearance without pulling in
//! the full canvas implementation. See [`super::scene`] for pan/zoom and [`super::draw`] for nodes.

use super::background_pattern::BackgroundPattern;
use super::graph_state::NodeState;
use super::pin::PinShape;
use super::wire::{WireLayer, WireStyle};
use egui::{Color32, CornerRadius, Frame, Margin, Stroke, Style, Vec2, epaint::Shadow, vec2};

/// Controls how header, pins, body and footer are placed in the node.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "egui-probe", derive(egui_probe::EguiProbe))]
pub enum NodeLayoutKind {
    /// Input pins, body and output pins are placed horizontally.
    /// With header on top and footer on bottom.
    ///
    /// +---------------------+
    /// |       Header        |
    /// +----+-----------+----+
    /// | In |           | Out|
    /// | In |   Body    | Out|
    /// | In |           | Out|
    /// | In |           |    |
    /// +----+-----------+----+
    /// |       Footer        |
    /// +---------------------+
    ///
    #[default]
    Coil,

    /// All elements are placed in vertical stack.
    /// Header is on top, then input pins, body, output pins and footer.
    ///
    /// +---------------------+
    /// |       Header        |
    /// +---------------------+
    /// | In                  |
    /// | In                  |
    /// | In                  |
    /// | In                  |
    /// +---------------------+
    /// |       Body          |
    /// +---------------------+
    /// |                 Out |
    /// |                 Out |
    /// |                 Out |
    /// +---------------------+
    /// |       Footer        |
    /// +---------------------+
    Sandwich,

    /// All elements are placed in vertical stack.
    /// Header is on top, then output pins, body, input pins and footer.
    ///
    /// +---------------------+
    /// |       Header        |
    /// +---------------------+
    /// |                 Out |
    /// |                 Out |
    /// |                 Out |
    /// +---------------------+
    /// |       Body          |
    /// +---------------------+
    /// | In                  |
    /// | In                  |
    /// | In                  |
    /// | In                  |
    /// +---------------------+
    /// |       Footer        |
    /// +---------------------+
    FlippedSandwich,
    // TODO: Add vertical layouts.
}

/// Controls how node elements are laid out.
///
///
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "egui-probe", derive(egui_probe::EguiProbe))]
pub struct NodeLayout {
    /// Controls method of laying out node elements.
    pub kind: NodeLayoutKind,

    /// Controls minimal height of pin rows.
    pub min_pin_row_height: f32,

    /// Controls how pin rows heights are set.
    /// If true, all pin rows will have the same height, matching the largest content.
    /// False by default.
    pub equal_pin_row_heights: bool,
}

impl NodeLayout {
    /// Creates new [`NodeLayout`] with `Coil` kind and flexible pin heights.
    #[must_use]
    #[inline]
    pub const fn coil() -> Self {
        NodeLayout {
            kind: NodeLayoutKind::Coil,
            min_pin_row_height: 0.0,
            equal_pin_row_heights: false,
        }
    }

    /// Creates new [`NodeLayout`] with `Sandwich` kind and flexible pin heights.
    #[must_use]
    #[inline]
    pub const fn sandwich() -> Self {
        NodeLayout {
            kind: NodeLayoutKind::Sandwich,
            min_pin_row_height: 0.0,
            equal_pin_row_heights: false,
        }
    }

    /// Creates new [`NodeLayout`] with `FlippedSandwich` kind and flexible pin heights.
    #[must_use]
    #[inline]
    pub const fn flipped_sandwich() -> Self {
        NodeLayout {
            kind: NodeLayoutKind::FlippedSandwich,
            min_pin_row_height: 0.0,
            equal_pin_row_heights: false,
        }
    }

    /// Returns new [`NodeLayout`] with same `kind` and specified pin heights.
    #[must_use]
    #[inline]
    pub const fn with_equal_pin_rows(self) -> Self {
        NodeLayout {
            kind: self.kind,
            min_pin_row_height: self.min_pin_row_height,
            equal_pin_row_heights: true,
        }
    }

    /// Returns new [`NodeLayout`] with same `kind` and specified minimum pin row height.
    #[must_use]
    #[inline]
    pub const fn with_min_pin_row_height(self, min_pin_row_height: f32) -> Self {
        NodeLayout {
            kind: self.kind,
            min_pin_row_height,
            equal_pin_row_heights: self.equal_pin_row_heights,
        }
    }
}

impl From<NodeLayoutKind> for NodeLayout {
    #[inline]
    fn from(kind: NodeLayoutKind) -> Self {
        NodeLayout {
            kind,
            min_pin_row_height: 0.0,
            equal_pin_row_heights: false,
        }
    }
}

impl Default for NodeLayout {
    #[inline]
    fn default() -> Self {
        NodeLayout::coil()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum OuterHeights<'a> {
    Flexible { rows: &'a [f32] },
    Matching { max: f32 },
    Tight,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Heights<'a> {
    rows: &'a [f32],
    outer: OuterHeights<'a>,
    min_outer: f32,
}

impl Heights<'_> {
    pub(crate) fn get(&self, idx: usize) -> (f32, f32) {
        let inner = match self.rows.get(idx) {
            Some(&value) => value,
            None => 0.0,
        };

        let outer = match &self.outer {
            OuterHeights::Flexible { rows } => match rows.get(idx) {
                Some(&outer) => outer.max(inner),
                None => inner,
            },
            OuterHeights::Matching { max } => max.max(inner),
            OuterHeights::Tight => inner,
        };

        (inner, outer.max(self.min_outer))
    }
}

impl NodeLayout {
    pub(crate) fn input_heights(self, state: &NodeState) -> Heights<'_> {
        let rows = state.input_heights().as_slice();

        let outer = match (self.kind, self.equal_pin_row_heights) {
            (NodeLayoutKind::Coil, false) => OuterHeights::Flexible {
                rows: state.output_heights().as_slice(),
            },
            (_, true) => {
                let mut max_height = 0.0f32;
                for &h in state.input_heights() {
                    max_height = max_height.max(h);
                }
                for &h in state.output_heights() {
                    max_height = max_height.max(h);
                }
                OuterHeights::Matching { max: max_height }
            }
            (_, false) => OuterHeights::Tight,
        };

        Heights {
            rows,
            outer,
            min_outer: self.min_pin_row_height,
        }
    }

    pub(crate) fn output_heights(self, state: &'_ NodeState) -> Heights<'_> {
        let rows = state.output_heights().as_slice();

        let outer = match (self.kind, self.equal_pin_row_heights) {
            (NodeLayoutKind::Coil, false) => OuterHeights::Flexible {
                rows: state.input_heights().as_slice(),
            },
            (_, true) => {
                let mut max_height = 0.0f32;
                for &h in state.input_heights() {
                    max_height = max_height.max(h);
                }
                for &h in state.output_heights() {
                    max_height = max_height.max(h);
                }
                OuterHeights::Matching { max: max_height }
            }
            (_, false) => OuterHeights::Tight,
        };

        Heights {
            rows,
            outer,
            min_outer: self.min_pin_row_height,
        }
    }
}

/// Controls style of node selection rect.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "egui-probe", derive(egui_probe::EguiProbe))]
pub struct SelectionStyle {
    /// Extra inset/outset for the selection outline vs. the node frame. Default is zero.
    pub margin: Margin,

    /// Rounding of selection rect.
    pub rounding: CornerRadius,

    /// Fill behind the selection outline. Use [`Color32::TRANSPARENT`] for a border only.
    pub fill: Color32,

    /// Outline of the selection rect.
    pub stroke: Stroke,
}

/// Controls how pins are placed in the node.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "egui-probe", derive(egui_probe::EguiProbe))]
pub enum PinPlacement {
    /// Pins are placed inside the node frame.
    #[default]
    Inside,

    /// Pins are placed on the edge of the node frame.
    Edge,

    /// Pins are placed outside the node frame.
    Outside {
        /// Margin between node frame and pins.
        margin: f32,
    },
}

/// Style for rendering NodeGraph.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "egui-probe", derive(egui_probe::EguiProbe))]
pub struct CanvasStyle {
    /// Controls how nodes are laid out.
    /// Defaults to [`NodeLayoutKind::Coil`].
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub node_layout: Option<NodeLayout>,

    /// Frame used to draw nodes.
    /// Defaults to [`Frame::window`] constructed from current ui's style.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            default,
            with = "serde_frame_option"
        )
    )]
    pub node_frame: Option<Frame>,

    /// Frame used to draw node headers.
    /// Defaults to the `node_frame` value without shadow and transparent fill.
    ///
    /// If set, it should not have shadow and fill should be either opaque of fully transparent
    /// unless layering of header fill color with node fill color is desired.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            default,
            with = "serde_frame_option"
        )
    )]
    pub header_frame: Option<Frame>,

    /// Blank space for dragging node by its header.
    /// Elements in the header are placed after this space.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub header_drag_space: Option<Vec2>,

    /// Whether nodes can be collapsed.
    /// If true, headers will have collapsing button.
    /// When collapsed, node will not show its pins, body and footer.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub collapsible: Option<bool>,

    /// Size of pins.
    #[cfg_attr(feature = "egui-probe", egui_probe(range = 0.0..))]
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub pin_size: Option<f32>,

    /// Default fill color for pins.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub pin_fill: Option<Color32>,

    /// Default stroke for pins.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub pin_stroke: Option<Stroke>,

    /// Shape of pins.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub pin_shape: Option<PinShape>,

    /// Placement of pins.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub pin_placement: Option<PinPlacement>,

    /// Width of wires.
    #[cfg_attr(feature = "egui-probe", egui_probe(range = 0.0..))]
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub wire_width: Option<f32>,

    /// Size of wire frame which controls curvature of wires.
    #[cfg_attr(feature = "egui-probe", egui_probe(range = 0.0..))]
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub wire_frame_size: Option<f32>,

    /// Whether to downscale wire frame when nodes are close.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub downscale_wire_frame: Option<bool>,

    /// Weather to upscale wire frame when nodes are far.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub upscale_wire_frame: Option<bool>,

    /// Controls default style of wires.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub wire_style: Option<WireStyle>,

    /// Layer where wires are rendered.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub wire_layer: Option<WireLayer>,

    /// Frame behind the graph. When `None`, uses [`Frame::canvas`] with fill `(19, 19, 19, 255)`.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            default,
            with = "serde_frame_option"
        )
    )]
    pub bg_frame: Option<Frame>,

    /// Background pattern.
    /// [`CanvasStyle::new`] sets this to [`BackgroundPattern::new`] (dots, 20×20, radius 0.5).
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub bg_pattern: Option<BackgroundPattern>,

    /// Stroke for background pattern.
    /// When `None`, width defaults to **0.30** and color to [`Style::visuals`] noninteractive stroke color.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub bg_pattern_stroke: Option<Stroke>,

    /// Minimum viewport scale that can be set.
    #[cfg_attr(feature = "egui-probe", egui_probe(range = 0.0..=1.0))]
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub min_scale: Option<f32>,

    /// Maximum viewport scale that can be set.
    #[cfg_attr(feature = "egui-probe", egui_probe(range = 1.0..))]
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub max_scale: Option<f32>,

    /// Enable centering by double click on background
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub centering: Option<bool>,

    /// Stroke for the outline around **selected nodes** (not the marquee drag rectangle).
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub select_stoke: Option<Stroke>,

    /// Stroke for the **marquee** (drag rectangle on empty canvas). Independent of [`Self::select_stoke`].
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub rect_select_stroke: Option<Stroke>,

    /// Fill for selection.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub select_fill: Option<Color32>,

    /// Flag to control how rect selection works.
    /// If set to true, only nodes fully contained in selection rect will be selected.
    /// If set to false, nodes intersecting with selection rect will be selected.
    pub select_rect_contained: Option<bool>,

    /// Style for node selection.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub select_style: Option<SelectionStyle>,

    /// Controls whether to show magnified text in crisp mode.
    /// This zooms UI style to max scale and scales down the scene.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub crisp_magnified_text: Option<bool>,

    /// Controls smoothness of wire curves.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    #[cfg_attr(
        feature = "egui-probe",
        egui_probe(range = 0.0f32..=10.0f32 by 0.05f32)
    )]
    pub wire_smoothness: Option<f32>,

    #[doc(hidden)]
    #[cfg_attr(feature = "egui-probe", egui_probe(skip))]
    #[cfg_attr(feature = "serde", serde(skip_serializing, default))]
    /// Do not access other than with .., here to emulate `#[non_exhaustive(pub)]`
    pub _non_exhaustive: (),
}

impl CanvasStyle {
    pub(crate) fn get_node_layout(&self) -> NodeLayout {
        self.node_layout.unwrap_or_default()
    }

    pub(crate) fn get_pin_size(&self, style: &Style) -> f32 {
        self.pin_size.unwrap_or(style.spacing.interact_size.y * 0.6)
    }

    pub(crate) fn get_pin_fill(&self, style: &Style) -> Color32 {
        self.pin_fill
            .unwrap_or(style.visuals.widgets.active.bg_fill)
    }

    pub(crate) fn get_pin_stroke(&self, style: &Style) -> Stroke {
        self.pin_stroke.unwrap_or_else(|| {
            Stroke::new(
                style.visuals.widgets.active.bg_stroke.width,
                style.visuals.widgets.active.bg_stroke.color,
            )
        })
    }

    pub(crate) fn get_pin_shape(&self) -> PinShape {
        self.pin_shape.unwrap_or(PinShape::Circle)
    }

    pub(crate) fn get_pin_placement(&self) -> PinPlacement {
        self.pin_placement.unwrap_or_default()
    }

    pub(crate) fn get_wire_width(&self, _style: &Style) -> f32 {
        self.wire_width.unwrap_or(3.0)
    }

    pub(crate) fn get_wire_frame_size(&self, style: &Style) -> f32 {
        self.wire_frame_size
            .unwrap_or_else(|| self.get_pin_size(style) * 3.0)
    }

    pub(crate) fn get_downscale_wire_frame(&self) -> bool {
        self.downscale_wire_frame.unwrap_or(true)
    }

    pub(crate) fn get_upscale_wire_frame(&self) -> bool {
        self.upscale_wire_frame.unwrap_or(false)
    }

    pub(crate) fn get_wire_style(&self) -> WireStyle {
        self.wire_style.unwrap_or(WireStyle::Bezier5)
    }

    pub(crate) fn get_wire_layer(&self) -> WireLayer {
        self.wire_layer.unwrap_or(WireLayer::BehindNodes)
    }

    pub(crate) fn get_header_drag_space(&self, style: &Style) -> Vec2 {
        self.header_drag_space
            .unwrap_or_else(|| vec2(style.spacing.icon_width, style.spacing.icon_width))
    }

    pub(crate) fn get_collapsible(&self) -> bool {
        self.collapsible.unwrap_or(true)
    }

    pub(crate) fn get_bg_frame(&self, style: &Style) -> Frame {
        self.bg_frame.unwrap_or_else(|| {
            let mut f = Frame::canvas(style);
            f.fill = Color32::from_rgba_unmultiplied(19, 19, 19, 255);
            f
        })
    }

    pub(crate) fn get_bg_pattern_stroke(&self, style: &Style) -> Stroke {
        self.bg_pattern_stroke.unwrap_or_else(|| {
            let base = style.visuals.widgets.noninteractive.bg_stroke;
            Stroke::new(0.30, base.color)
        })
    }

    pub(crate) fn get_min_scale(&self) -> f32 {
        self.min_scale.unwrap_or(0.2)
    }

    pub(crate) fn get_max_scale(&self) -> f32 {
        self.max_scale.unwrap_or(2.0)
    }

    /// Rounding aligned with the selection outline when `select_style` is set; if `None`, uniform window theme.
    /// Used only when [`CanvasStyle::node_frame`] is `None` so explicit frames keep per-corner edits from the UI.
    #[inline]
    pub(crate) fn default_chrome_corner_rounding(&self, style: &Style) -> CornerRadius {
        self.select_style
            .map(|s| s.rounding)
            .unwrap_or_else(|| uniform_window_corner_radius(style))
    }

    pub(crate) fn get_node_frame(&self, style: &Style) -> Frame {
        match self.node_frame {
            Some(f) => f,
            None => {
                let mut f = Frame::window(style);
                f.corner_radius = self.default_chrome_corner_rounding(style);
                f
            }
        }
    }

    pub(crate) fn get_header_frame(&self, style: &Style) -> Frame {
        match self.header_frame {
            Some(h) => h,
            None => self.get_node_frame(style).shadow(Shadow::NONE),
        }
    }

    pub(crate) fn get_centering(&self) -> bool {
        self.centering.unwrap_or(true)
    }

    pub(crate) fn get_select_stroke(&self, _style: &Style) -> Stroke {
        self.select_stoke.unwrap_or_else(default_selection_stroke)
    }

    pub(crate) fn get_rect_select_stroke(&self) -> Stroke {
        self.rect_select_stroke
            .unwrap_or_else(default_rect_selection_stroke)
    }

    pub(crate) fn get_select_fill(&self, _style: &Style) -> Color32 {
        self.select_fill.unwrap_or_else(default_selection_fill)
    }

    pub(crate) fn get_select_rect_contained(&self) -> bool {
        self.select_rect_contained.unwrap_or(false)
    }

    pub(crate) fn get_select_style(&self, style: &Style) -> SelectionStyle {
        self.select_style.unwrap_or_else(|| {
            let rounding = self.default_chrome_corner_rounding(style);
            SelectionStyle {
                margin: Margin::same(2),
                rounding,
                // Border only; marquee drag-rect still uses [`get_select_fill`].
                fill: Color32::TRANSPARENT,
                stroke: self.get_select_stroke(style),
            }
        })
    }

    pub(crate) fn get_crisp_magnified_text(&self) -> bool {
        self.crisp_magnified_text.unwrap_or(false)
    }

    pub(crate) fn get_wire_smoothness(&self) -> f32 {
        self.wire_smoothness.unwrap_or(0.0)
    }
}

impl egui_scale::EguiScale for WireStyle {
    #[inline(always)]
    fn scale(&mut self, scale: f32) {
        match self {
            WireStyle::Line | WireStyle::Bezier3 | WireStyle::Bezier5 => {}
            WireStyle::AxisAligned { corner_radius } => {
                corner_radius.scale(scale);
            }
        }
    }
}

impl egui_scale::EguiScale for SelectionStyle {
    #[inline(always)]
    fn scale(&mut self, scale: f32) {
        self.margin.scale(scale);
        self.rounding.scale(scale);
        self.stroke.scale(scale);
    }
}

impl egui_scale::EguiScale for PinPlacement {
    fn scale(&mut self, scale: f32) {
        if let PinPlacement::Outside { margin } = self {
            margin.scale(scale);
        }
    }
}

impl egui_scale::EguiScale for BackgroundPattern {
    fn scale(&mut self, scale: f32) {
        if let BackgroundPattern::Grid(grid) = self {
            grid.spacing *= scale;
            grid.phase *= scale;
            grid.dot_radius *= scale;
        }
    }
}

impl egui_scale::EguiScale for CanvasStyle {
    fn scale(&mut self, scale: f32) {
        self.node_frame.scale(scale);
        self.header_frame.scale(scale);
        self.header_drag_space.scale(scale);
        self.pin_size.scale(scale);
        self.pin_stroke.scale(scale);
        self.pin_placement.scale(scale);
        self.wire_width.scale(scale);
        self.wire_frame_size.scale(scale);
        self.wire_style.scale(scale);
        self.bg_frame.scale(scale);
        self.bg_pattern.scale(scale);
        self.bg_pattern_stroke.scale(scale);
        self.min_scale.scale(scale);
        self.max_scale.scale(scale);
        self.select_stoke.scale(scale);
        self.rect_select_stroke.scale(scale);
        self.select_style.scale(scale);
    }
}

#[cfg(feature = "serde")]
mod serde_frame_option {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    pub struct Frame {
        pub inner_margin: egui::Margin,
        pub outer_margin: egui::Margin,
        pub rounding: egui::CornerRadius,
        pub shadow: egui::epaint::Shadow,
        pub fill: egui::Color32,
        pub stroke: egui::Stroke,
    }

    #[allow(clippy::ref_option)]
    pub fn serialize<S>(frame: &Option<egui::Frame>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match frame {
            Some(frame) => Frame {
                inner_margin: frame.inner_margin,
                outer_margin: frame.outer_margin,
                rounding: frame.corner_radius,
                shadow: frame.shadow,
                fill: frame.fill,
                stroke: frame.stroke,
            }
            .serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<egui::Frame>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let frame_opt = Option::<Frame>::deserialize(deserializer)?;
        Ok(frame_opt.map(|frame| egui::Frame {
            inner_margin: frame.inner_margin,
            outer_margin: frame.outer_margin,
            corner_radius: frame.rounding,
            shadow: frame.shadow,
            fill: frame.fill,
            stroke: frame.stroke,
        }))
    }
}

/// Single radius on all corners, derived from [`Style::visuals`] window rounding.
///
/// Used when [`CanvasStyle::select_style`] is `None` so fallback selection and node chrome still match.
#[inline]
fn uniform_window_corner_radius(style: &Style) -> CornerRadius {
    let w = style.visuals.window_corner_radius;
    CornerRadius::same(w.nw.max(w.ne).max(w.sw).max(w.se))
}

/// Default stroke for **selected node** outline when [`CanvasStyle::select_stoke`] is unset.
#[inline]
pub(crate) fn default_selection_stroke() -> Stroke {
    Stroke::new(3.0, Color32::from_rgba_unmultiplied(255, 255, 255, 125))
}

/// Default stroke for the **marquee** (drag rectangle) when [`CanvasStyle::rect_select_stroke`] is unset.
#[inline]
pub(crate) fn default_rect_selection_stroke() -> Stroke {
    Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 125))
}

/// Default fill for marquee rectangle when [`CanvasStyle::select_fill`] is unset.
#[inline]
pub(crate) fn default_selection_fill() -> Color32 {
    Color32::from_rgba_unmultiplied(255, 255, 255, 25)
}

#[inline]
fn default_selection_rounding() -> CornerRadius {
    CornerRadius::same(4)
}

impl CanvasStyle {
    /// Creates new [`CanvasStyle`] filled with default values.
    ///
    /// Selection uses explicit stroke, margins, and rounding so the first frame matches later frames
    /// (no dependency on theme-only fallbacks before the first style sync).
    #[must_use]
    pub fn new() -> Self {
        let selection_stroke = default_selection_stroke();
        let selection_rounding = default_selection_rounding();
        CanvasStyle {
            node_layout: None,
            pin_size: None,
            pin_fill: None,
            pin_stroke: None,
            pin_shape: None,
            pin_placement: None,
            wire_width: None,
            wire_frame_size: None,
            downscale_wire_frame: None,
            upscale_wire_frame: None,
            wire_style: None,
            wire_layer: None,
            header_drag_space: None,
            collapsible: None,

            bg_frame: None,
            bg_pattern: Some(BackgroundPattern::new()),
            bg_pattern_stroke: None,

            min_scale: None,
            max_scale: None,
            node_frame: None,
            header_frame: None,
            centering: None,
            select_stoke: Some(selection_stroke),
            rect_select_stroke: Some(default_rect_selection_stroke()),
            select_fill: Some(default_selection_fill()),
            select_rect_contained: None,
            select_style: Some(SelectionStyle {
                margin: Margin::same(2),
                rounding: selection_rounding,
                fill: Color32::TRANSPARENT,
                stroke: selection_stroke,
            }),
            crisp_magnified_text: None,
            wire_smoothness: None,

            _non_exhaustive: (),
        }
    }

    /// Canvas style matching the bundled style panel ([`crate::ui::canvas_style_panel::canvas_style_controls_ui`]).
    ///
    /// [`CanvasStyle::new`] stays the neutral API default; use this when you want explicit pin colors,
    /// zoom limits, and wire options aligned with the helper panel without duplicating `Some(…)` in the app.
    #[must_use]
    pub fn editor_tuned() -> Self {
        let mut s = Self::new();
        s.node_layout = Some(NodeLayout::coil());
        s.collapsible = Some(true);
        s.pin_size = Some(8.0);
        s.pin_fill = Some(Color32::from_rgba_unmultiplied(70, 70, 70, 255));
        s.pin_stroke = Some(Stroke::new(1.5, Color32::WHITE));
        s.pin_shape = Some(PinShape::Circle);
        s.pin_placement = Some(PinPlacement::Edge);
        s.wire_width = Some(3.0);
        s.wire_frame_size = Some(32.0);
        s.downscale_wire_frame = Some(true);
        s.upscale_wire_frame = Some(false);
        s.wire_style = Some(WireStyle::Bezier5);
        s.wire_layer = Some(WireLayer::BehindNodes);
        s.bg_pattern = Some(BackgroundPattern::new());
        s.min_scale = Some(1.0);
        s.max_scale = Some(1.10);
        s.centering = Some(true);
        s.wire_smoothness = Some(0.0);
        s
    }
}

impl Default for CanvasStyle {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
