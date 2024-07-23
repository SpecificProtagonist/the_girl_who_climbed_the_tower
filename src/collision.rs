use std::ops::Index;

use bevy::{
    math::{ivec2, vec2},
    prelude::*,
};

use crate::CELL_SIZE;

#[derive(Resource)]
pub struct Level {
    pub grid: Vec<i32>,
}

impl Level {
    pub fn collision(&self, pos: Vec2, radius: f32, mut movement: Vec2, flying: bool) -> Vec2 {
        let min = ((pos.min(pos + movement) - radius) / CELL_SIZE - 0.5)
            .round()
            .as_ivec2();
        let max = ((pos.max(pos + movement) + radius) / CELL_SIZE + 0.5)
            .round()
            .as_ivec2();
        for x in min.x..=max.x {
            for y in min.y..=max.y {
                if !(0..16).contains(&x) | !(0..16).contains(&y) {
                    return Vec2::ZERO;
                }
                let cell = self[ivec2(x, y)];
                if (cell == 0) | ((cell == 2) & !flying) {
                    movement = with_cell(ivec2(x, y), pos, radius, movement)
                }
            }
        }
        movement
    }
}

impl Index<IVec2> for Level {
    type Output = i32;

    fn index(&self, index: IVec2) -> &Self::Output {
        &self.grid[(index.x + (15 - index.y) * 16) as usize]
    }
}

pub fn with_cell(cell: IVec2, pos: Vec2, radius: f32, vel: Vec2) -> Vec2 {
    let rel = pos - cell.as_vec2() * CELL_SIZE;

    let mut movement = vel;
    let mut progress = 1.;

    let progress_x = if vel.x > 0. {
        (0. - rel.x - radius) / vel.x
    } else if vel.x < 0. {
        (CELL_SIZE - rel.x + radius) / vel.x
    } else {
        f32::INFINITY
    };
    let progress_x_contact_y = rel.y + vel.y * progress_x;
    if (progress_x > 0.)
        & (progress_x < progress)
        & (0. ..CELL_SIZE).contains(&progress_x_contact_y)
    {
        movement.x = (progress_x * vel.x.abs() - 0.1) * vel.x.signum();
        progress = progress_x;
    }
    let progress_y = if vel.y > 0. {
        (0. - rel.y - radius) / vel.y
    } else if vel.y < 0. {
        (CELL_SIZE - rel.y + radius) / vel.y
    } else {
        f32::INFINITY
    };
    let progress_y_contact_x = rel.x + vel.x * progress_y;
    if (progress_y > 0.)
        & (progress_y < progress)
        & (0. ..CELL_SIZE).contains(&progress_y_contact_x)
    {
        movement.y = (progress_y * vel.y.abs() - 0.1) * vel.y.signum();
    }

    for corner in [vec2(0., 0.), vec2(0., 1.), vec2(1., 0.), vec2(1., 1.)] {
        movement = with_ball(corner * CELL_SIZE, 0., rel, radius, movement);
    }

    movement
}

pub fn with_ball(ball_pos: Vec2, ball_radius: f32, pos: Vec2, radius: f32, vel: Vec2) -> Vec2 {
    let rel = pos - ball_pos;
    let radius = ball_radius + radius;
    let mut movement = vel;
    if let Some(dist) = ray(-rel, radius, vel) {
        let movement_to_collision = dist * vel.normalize();
        let leftover = vel * (1. - dist / vel.length());
        let additional = leftover.project_onto(rel.perp());
        movement = movement_to_collision + additional;
    }
    movement
}

fn ray(ball_pos: Vec2, radius: f32, cast_to: Vec2) -> Option<f32> {
    let projected = (ball_pos).project_onto(cast_to);
    let towards = cast_to.signum() == projected.signum();
    let projected_distance = (projected - ball_pos).length();
    if towards & (projected_distance < radius) {
        let within_circle = (radius.powi(2) - projected_distance.powi(2)).sqrt();
        let until_circle = projected.length() - within_circle;
        (until_circle < cast_to.length()).then_some(until_circle)
    } else {
        None
    }
}
