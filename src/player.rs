use bevy::{
    math::{ivec2, vec2, vec3},
    prelude::*,
};

use crate::{Handles, Vel, CELL_SIZE};

#[derive(Component, Default)]
pub struct Player {
    walk_ani: f32,
    shoot_cooldown: f32,
}

#[derive(Component)]
pub struct Bullet {
    _sprite: Entity,
}

pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player: Query<(
        &mut Transform,
        &mut Vel,
        &mut Player,
        &mut Sprite,
        &mut Handle<Image>,
    )>,
    tex_ass: Res<Handles>,
    mut gizmos: Gizmos,
) {
    let Ok((mut pos, mut velocity, mut player, mut sprite, mut tex)) = player.get_single_mut()
    else {
        return;
    };
    let mut dir = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::KeyA) {
        dir -= Vec2::X;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        dir += Vec2::X;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        dir -= Vec2::Y;
    }
    if keyboard_input.pressed(KeyCode::KeyW) {
        dir += Vec2::Y;
    }
    let speed = 30.;
    let vel = dir.normalize_or_zero() * speed;
    let mut movement = vel * time.delta_seconds();
    gizmos.rect_2d(
        vec2(10., 10.) * CELL_SIZE,
        0.,
        Vec2::splat(CELL_SIZE),
        Color::srgb(1., 1., 0.),
    );
    let collider_pos = vec2(10., 10.) * CELL_SIZE;
    // gizmos.circle_2d(collider_pos, 7., Color::srgb(0., 1., 0.));
    gizmos.circle_2d(pos.translation.xy(), 3., Color::srgb(0., 1., 0.));
    if vel != Vec2::ZERO {
        // crate::collision::with_ball(
        //     &mut gizmos,
        //     collider_pos,
        //     7.,
        //     pos.translation.xy(),
        //     3.,
        //     movement,
        // );
        movement = crate::collision::with_cell(
            &mut gizmos,
            ivec2(10, 10),
            pos.translation.xy(),
            3.,
            movement,
        );
    }
    velocity.0 = vel;
    pos.translation += (movement).extend(0.);

    if dir != Vec2::ZERO {
        player.walk_ani += time.delta_seconds();
        if player.walk_ani > 0.6 {
            player.walk_ani -= 0.6;
        }
    }
    let index = if player.walk_ani < 0.3 { 0 } else { 1 };
    if dir.y < 0. {
        *tex = tex_ass.player_down[index].clone();
    } else if dir.y > 0. {
        *tex = tex_ass.player_up[index].clone();
    } else if dir.x < 0. {
        *tex = tex_ass.player_side[index].clone();
        sprite.flip_x = true;
    } else if dir.x > 0. {
        *tex = tex_ass.player_side[index].clone();
        sprite.flip_x = false;
    }
}

pub fn player_shoot(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player: Query<(&Transform, &Vel, &mut Player)>,
    handles: Res<Handles>,
) {
    let Ok((pos, player_vel, mut player)) = player.get_single_mut() else {
        return;
    };
    player.shoot_cooldown -= time.delta_seconds();
    if player.shoot_cooldown > 0. {
        return;
    }

    let mut dir = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        dir -= Vec2::X;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        dir += Vec2::X;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        dir -= Vec2::Y;
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        dir += Vec2::Y;
    }

    if dir == Vec2::ZERO {
        return;
    }
    let dir = dir.normalize();
    let vel = Dir2::new(dir + player_vel.0 * 0.005).unwrap() * 180.;

    player.shoot_cooldown = 0.4;
    let mut sprite = Entity::PLACEHOLDER;
    commands
        .spawn(())
        .with_children(|b| {
            sprite = b
                .spawn((SpriteBundle {
                    texture: handles.bullet.clone(),
                    transform: Transform {
                        translation: vec3(0., 12., 0.),
                        rotation: Quat::from_rotation_z(dir.to_angle()),
                        ..default()
                    },
                    ..default()
                },))
                .id();
        })
        .insert((
            Transform::from_translation(pos.translation + dir.extend(0.) * 5.),
            Vel(vel),
            Bullet { _sprite: sprite },
            GlobalTransform::default(),
            InheritedVisibility::default(),
        ));
}

pub fn move_bullets(mut bullets: Query<(&mut Transform, &Vel), With<Bullet>>, time: Res<Time>) {
    for (mut pos, vel) in &mut bullets {
        pos.translation += vel.extend(0.) * time.delta_seconds();
    }
}
