use bevy::{
    math::{vec2, NormedVectorSpace},
    prelude::*,
};

use crate::CELL_SIZE;

pub fn with_cell(gizmos: &mut Gizmos, cell: IVec2, pos: Vec2, radius: f32, vel: Vec2) -> Vec2 {
    let rel = pos - cell.as_vec2() * CELL_SIZE;

    let mut movement = vel;
    let mut progress = 1.;

    let half_size = 0.5 * CELL_SIZE;

    let progress_x = if vel.x > 0. {
        (-half_size - rel.x - radius) / vel.x
    } else if vel.x < 0. {
        (half_size - rel.x + radius) / vel.x
    } else {
        f32::INFINITY
    };
    let progress_x_contact_y = rel.y + vel.y * progress_x;
    if (progress_x > 0.)
        & (progress_x < progress)
        & (-half_size..half_size).contains(&progress_x_contact_y)
    {
        movement.x = (progress_x * vel.x.abs() - 0.1) * vel.x.signum();
        progress = progress_x;
    }
    let progress_y = if vel.y > 0. {
        (-half_size - rel.y - radius) / vel.y
    } else if vel.y < 0. {
        (half_size - rel.y + radius) / vel.y
    } else {
        f32::INFINITY
    };
    let progress_y_contact_x = rel.x + vel.x * progress_y;
    if (progress_y > 0.)
        & (progress_y < progress)
        & (-half_size..half_size).contains(&progress_y_contact_x)
    {
        movement.y = (progress_y * vel.y.abs() - 0.1) * vel.y.signum();
        // progress = progress_y;
    }

    for corner in [vec2(1., 1.), vec2(1., -1.), vec2(-1., 1.), vec2(-1., -1.)] {
        // let rel = rel + corner * half_size;
        movement = with_ball(
            gizmos,
            cell.as_vec2() * CELL_SIZE + corner * half_size,
            0.,
            pos,
            radius,
            movement,
        );
    }

    movement
}

pub fn with_ball(
    gizmos: &mut Gizmos,
    ball_pos: Vec2,
    ball_radius: f32,
    pos: Vec2,
    radius: f32,
    vel: Vec2,
) -> Vec2 {
    let rel = pos - ball_pos;
    let radius = ball_radius + radius;
    let mut movement = vel;
    if let Some(dist) = ray(gizmos, pos, -rel, radius, vel) {
        let movement_to_collision = dist * vel.normalize();
        let leftover = vel * (1. - dist / vel.length());
        let additional = leftover.project_onto(rel.perp());
        movement = movement_to_collision + additional;
    }
    movement
}

fn ray(
    gizmos: &mut Gizmos,
    tmp_pos: Vec2,
    ball_pos: Vec2,
    radius: f32,
    cast_to: Vec2,
) -> Option<f32> {
    let projected = (ball_pos).project_onto(cast_to);
    let towards = cast_to.signum() == projected.signum();
    let projected_distance = (projected - ball_pos).length();
    if towards & (projected_distance < radius) {
        gizmos.circle_2d(tmp_pos + projected, 1.5, Color::srgb(1., 0.5, 0.));
        let within_circle = (radius.powi(2) - projected_distance.powi(2)).sqrt();
        let until_circle = projected.length() - within_circle;
        gizmos.line_2d(
            tmp_pos + projected,
            tmp_pos + projected - cast_to.normalize() * within_circle,
            Color::srgb(0., 0., 1.),
        );
        gizmos.line_2d(
            tmp_pos,
            tmp_pos + cast_to.normalize() * until_circle,
            Color::srgb(0., 1., 1.),
        );
        (until_circle < cast_to.length()).then_some(until_circle)
    } else {
        gizmos.circle_2d(tmp_pos + projected, 1.5, Color::srgb(1., 1., 0.));
        None
    }
}
