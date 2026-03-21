use std::sync::Arc;

use egui::{Color32, Stroke, Style};

use crate::ui::nodes_canvas::{BackgroundPattern, Grid, CanvasStyle};

/// Hooks for strokes similar in spirit to egui_graphs-style customization.
pub trait NodeStyleHook: Send + Sync {
    fn stroke(
        &self,
        selected: bool,
        dragged: bool,
        node_color: Option<Color32>,
        default: Stroke,
        egui_style: &Style,
    ) -> Stroke;
}

/// Default: emphasize stroke when the node is selected (using egui selection colors).
pub struct DefaultNodeStyleHook;

impl NodeStyleHook for DefaultNodeStyleHook {
    fn stroke(
        &self,
        selected: bool,
        dragged: bool,
        _node_color: Option<Color32>,
        default: Stroke,
        egui_style: &Style,
    ) -> Stroke {
        let _ = dragged;
        if selected {
            Stroke::new(
                default.width.max(1.5),
                egui_style.visuals.selection.stroke.color,
            )
        } else {
            default
        }
    }
}

pub trait EdgeStyleHook: Send + Sync {
    fn stroke(
        &self,
        selected: bool,
        order: usize,
        default: Stroke,
        egui_style: &Style,
    ) -> Stroke;
}

pub struct DefaultEdgeStyleHook;

impl EdgeStyleHook for DefaultEdgeStyleHook {
    fn stroke(
        &self,
        selected: bool,
        order: usize,
        default: Stroke,
        egui_style: &Style,
    ) -> Stroke {
        let _ = order;
        if selected {
            Stroke::new(default.width, egui_style.visuals.selection.stroke.color)
        } else {
            default
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BackgroundStyle {
    pub dim: f32,
}

impl Default for BackgroundStyle {
    fn default() -> Self {
        Self { dim: 1.0 }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GridSettings {
    pub spacing: egui::Vec2,
    pub angle: f32,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            spacing: egui::vec2(50.0, 50.0),
            angle: 0.0,
        }
    }
}

/// User-facing style: NodeGraph draw parameters plus hooks used by [`crate::ui::editor::shell_viewer::NodesShellViewer`].
#[derive(Clone)]
pub struct NodesStyle {
    pub node_style: Arc<dyn NodeStyleHook>,
    pub edge_style: Arc<dyn EdgeStyleHook>,
    pub background_style: BackgroundStyle,
    pub grid: GridSettings,
    pub canvas: CanvasStyle,
}

impl Default for NodesStyle {
    fn default() -> Self {
        Self::new()
    }
}

impl NodesStyle {
    pub fn new() -> Self {
        Self {
            node_style: Arc::new(DefaultNodeStyleHook),
            edge_style: Arc::new(DefaultEdgeStyleHook),
            background_style: BackgroundStyle::default(),
            grid: GridSettings::default(),
            canvas: CanvasStyle::new(),
        }
    }

    pub fn with_node_style(mut self, hook: impl NodeStyleHook + 'static) -> Self {
        self.node_style = Arc::new(hook);
        self
    }

    pub fn with_edge_style(mut self, hook: impl EdgeStyleHook + 'static) -> Self {
        self.edge_style = Arc::new(hook);
        self
    }

    /// Applies grid settings from [`Self::grid`] into [`Self::canvas`] when the pattern is a grid.
    pub fn sync_grid_into_canvas(&mut self) {
        let g = self.grid;
        match &mut self.canvas.bg_pattern {
            Some(BackgroundPattern::Grid(grid)) => {
                grid.spacing = g.spacing;
                grid.angle = g.angle;
            }
            None => {
                self.canvas.bg_pattern = Some(BackgroundPattern::Grid(Grid::new(g.spacing, g.angle)));
            }
            Some(BackgroundPattern::NoPattern) => {}
        }
    }

    /// Node graph style passed to [`NodesCanvas`](crate::ui::nodes_canvas::NodesCanvas). Hooks that need egui’s global style
    /// run in the shell viewer; this is the copied [`CanvasStyle`] snapshot.
    pub fn to_canvas_style(&self) -> CanvasStyle {
        self.canvas
    }
}
