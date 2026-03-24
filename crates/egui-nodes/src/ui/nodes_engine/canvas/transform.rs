//! Shared 2D scene transforms for [`super::graph_state::CanvasState`] and [`super::scene`].

use egui::{Pos2, Rect, emath::TSTransform};

#[inline]
pub(crate) fn clamp_scale(
    to_global: &mut TSTransform,
    min_scale: f32,
    max_scale: f32,
    ui_rect: Rect,
) {
    let mut min_scale = min_scale;
    let mut max_scale = max_scale;

    // Keep scale bounds valid and finite even if caller-provided style values are malformed.
    if !min_scale.is_finite() || min_scale <= 0.0 {
        min_scale = 0.2;
    }
    if !max_scale.is_finite() || max_scale < min_scale {
        max_scale = min_scale;
    }

    if to_global.scaling.is_finite()
        && to_global.scaling >= min_scale
        && to_global.scaling <= max_scale
    {
        return;
    }

    let current_scaling = if to_global.scaling.is_finite() {
        to_global.scaling
    } else {
        min_scale
    };
    let new_scaling = current_scaling.clamp(min_scale, max_scale);
    *to_global = scale_transform_around(to_global, new_scaling, ui_rect.center());
}

#[inline]
#[must_use]
pub(crate) fn transform_matching_points(from: Pos2, to: Pos2, scaling: f32) -> TSTransform {
    TSTransform {
        scaling,
        translation: to.to_vec2() - from.to_vec2() * scaling,
    }
}

#[inline]
#[must_use]
pub(crate) fn scale_transform_around(
    transform: &TSTransform,
    scaling: f32,
    point: Pos2,
) -> TSTransform {
    let base_scaling = if transform.scaling.is_finite() && transform.scaling.abs() > f32::EPSILON {
        transform.scaling
    } else {
        1.0
    };
    let from = (point - transform.translation) / base_scaling;
    transform_matching_points(from, point, scaling)
}
