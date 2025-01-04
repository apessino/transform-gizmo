use glam::DVec3;

use crate::math::{round_to_interval, world_to_screen, Pos2};

use crate::subgizmo::common::{
    draw_arrow, draw_circle, draw_plane, gizmo_color, outer_circle_radius, pick_arrow, pick_circle,
    pick_plane,
};
use crate::subgizmo::{common::TransformKind, SubGizmoConfig, SubGizmoKind};
use crate::{gizmo::Ray, GizmoDirection, GizmoDrawData, GizmoMode, GizmoResult};

pub(crate) type ScaleSubGizmo = SubGizmoConfig<Scale>;

#[derive(Debug, Copy, Clone, Hash)]
pub(crate) struct ScaleParams {
    pub mode: GizmoMode,
    pub direction: GizmoDirection,
    pub transform_kind: TransformKind,
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct ScaleState {
    start_pos: emath::Pos2,
    start_delta: f64,
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct Scale;

impl SubGizmoKind for Scale {
    type Params = ScaleParams;
    type State = ScaleState;

    fn pick(subgizmo: &mut ScaleSubGizmo, ray: Ray) -> Option<f64> {
        let pick_result = match (subgizmo.transform_kind, subgizmo.direction) {
            (TransformKind::Plane, GizmoDirection::View) => pick_circle(
                &subgizmo.config,
                ray,
                outer_circle_radius(&subgizmo.config),
                false,
            ),
            (TransformKind::Plane, _) => pick_plane(&subgizmo.config, ray, subgizmo.direction),
            (TransformKind::Axis, _) => {
                pick_arrow(&subgizmo.config, ray, subgizmo.direction, subgizmo.mode)
            }
        };

        let start_pos = ray.screen_pos;
        let start_delta = distance_from_origin_2d(subgizmo, start_pos)?;

        subgizmo.opacity = pick_result.visibility as _;

        subgizmo.state.start_pos = start_pos;
        subgizmo.state.start_delta = start_delta;

        if pick_result.picked {
            Some(pick_result.t)
        } else {
            None
        }
    }

    /// $AGP: modified with our own scaling that *ONLY SUPPORTS UNIFORM SCALING* but that
    /// feels much better in use
    fn update(subgizmo: &mut ScaleSubGizmo, ray: Ray) -> Option<GizmoResult> {
        let origin = origin_2d(subgizmo)?;

        let dir = (subgizmo.state.start_pos - origin).normalized(); // direction from gizmo center to drag origin
        let change = ray.screen_pos - subgizmo.state.start_pos; // change in position from start point
        let dot = dir.x * change.x + dir.y * change.y;
        let mut delta = scale_map(dot / (subgizmo.state.start_pos - origin).length());

        if subgizmo.config.snapping {
            delta = round_to_interval(delta as f64, subgizmo.config.snap_scale as f64) as f32;
        }

        let scale = DVec3::ONE * delta as f64;

        Some(GizmoResult::Scale {
            total: scale.into(),
        })
    }

    fn draw(subgizmo: &ScaleSubGizmo) -> GizmoDrawData {
        match (subgizmo.transform_kind, subgizmo.direction) {
            (TransformKind::Axis, _) => draw_arrow(
                &subgizmo.config,
                subgizmo.opacity,
                subgizmo.focused,
                subgizmo.direction,
                subgizmo.mode,
            ),
            (TransformKind::Plane, GizmoDirection::View) => draw_circle(
                &subgizmo.config,
                gizmo_color(&subgizmo.config, subgizmo.focused, subgizmo.direction),
                outer_circle_radius(&subgizmo.config),
                false,
            ),
            (TransformKind::Plane, _) => draw_plane(
                &subgizmo.config,
                subgizmo.opacity,
                subgizmo.focused,
                subgizmo.direction,
            ),
        }
    }
}

fn origin_2d<T: SubGizmoKind>(subgizmo: &SubGizmoConfig<T>) -> Option<Pos2> {
    Some(world_to_screen(
        subgizmo.config.viewport,
        subgizmo.config.mvp,
        DVec3::new(0.0, 0.0, 0.0),
    )?)
}
fn distance_from_origin_2d<T: SubGizmoKind>(
    subgizmo: &SubGizmoConfig<T>,
    cursor_pos: Pos2,
) -> Option<f64> {
    Some(cursor_pos.distance(origin_2d(subgizmo)?) as f64)
}
fn scale_map(d: f32) -> f32 {
    (d + f32::sqrt(d * d + 4.0)) * 0.5
}
