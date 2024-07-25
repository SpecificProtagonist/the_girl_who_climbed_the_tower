use bevy::{
    math::{ivec2, vec2, vec3},
    prelude::*,
};
use rand::prelude::*;

use crate::{
    aseprite::Animation,
    collision::grid_collision,
    level::{Tile, Tiles, CELL_SIZE},
    player::{Bullet, Player},
    Clearable, Cycle, Handles, Layer, Vel,
};

static FLOATER_SIZE: f32 = 5.;

#[derive(Component)]
pub struct Enemy {
    pub health: f32,
    pub size: f32,
}

#[derive(Component)]
pub struct HurtIndicator {
    pub last_hit: f32,
    pub occluder: Entity,
}

#[derive(Component)]
pub struct Floater {
    movement_timer: f32,
}

#[derive(Component)]
pub struct Spawner {
    timer: f32,
    enemy: Entity,
    summon_occluder: Entity,
}

impl Spawner {
    pub fn create(pos: Vec2, delay: f32) -> impl Bundle {
        (
            Self {
                timer: delay + 1.25,
                enemy: Entity::PLACEHOLDER,
                summon_occluder: Entity::PLACEHOLDER,
            },
            Transform::from_translation(pos.extend(0.)),
        )
    }
}

pub fn spawn_enemies(mut commands: Commands, tiles: Res<Tiles>, cycle: Res<Cycle>) {
    // Don't spawn enemies in the very first room
    if (cycle.current_room == 0) & (cycle.cycle == 0) {
        return;
    }

    let mut floor = Vec::new();
    for x in 0..16 {
        for y in 0..16 {
            // Free & not occluded from vision
            if (tiles[ivec2(x, y)] == Tile::Floor)
                & (y < 2
                    || (tiles[ivec2(x, y - 1)] != Tile::Wall)
                        & (tiles[ivec2(x, y - 2)] != Tile::Wall))
            {
                floor.push(ivec2(x, y));
            }
        }
    }
    for i in 0..4 + cycle.cycle {
        let delay = 2. + 0.3 * i as f32;
        let tile_center =
            (floor.choose(&mut thread_rng()).unwrap().as_vec2() + vec2(0.5, 0.5)) * CELL_SIZE;
        let offset = grid_collision(
            &tiles,
            tile_center,
            FLOATER_SIZE,
            vec2(
                thread_rng().gen_range(-0.5..0.5),
                thread_rng().gen_range(-0.5..0.5),
            ) * CELL_SIZE,
            false,
        );
        commands.spawn((Spawner::create(tile_center + offset, delay), Clearable));
    }
}

pub fn spawners(
    mut commands: Commands,
    mut spawners: Query<(Entity, &Transform, &mut Spawner)>,
    mut sprites: Query<&mut Sprite>,
    time: Res<Time>,
    handles: Res<Handles>,
) {
    for (entity, trans, mut spawner) in &mut spawners {
        let step = spawner.timer - time.delta_seconds()..spawner.timer;
        spawner.timer -= time.delta_seconds();
        if step.contains(&1.2) {
            commands.spawn((
                Layer(-0.9),
                SpriteBundle {
                    transform: *trans,
                    ..default()
                },
                Animation::new(handles.summon.clone()),
            ));
        }
        if step.contains(&1.0) {
            commands.spawn(AudioBundle {
                source: handles.sfx_summon.clone(),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: bevy::audio::Volume::new(0.5),
                    ..default()
                },
            });
        }
        if step.contains(&0.7) {
            spawner.enemy = commands
                .spawn_empty()
                .with_children(|b| {
                    // TODO: modify the sprite shader instead
                    spawner.summon_occluder = b
                        .spawn((SpriteBundle {
                            sprite: Sprite {
                                anchor: bevy::sprite::Anchor::BottomCenter,
                                color: Color::hsla(300., 1., 0.85, 1.),
                                ..default()
                            },
                            texture: handles.enemy_summon.clone(),
                            transform: Transform::from_xyz(0., 0., 0.0001),
                            ..default()
                        },))
                        .id();
                })
                .insert((
                    Clearable,
                    Layer(0.),
                    Vel(Vec2::ZERO),
                    SpriteBundle {
                        sprite: Sprite {
                            anchor: bevy::sprite::Anchor::BottomCenter,
                            ..default()
                        },
                        texture: handles.enemy.clone(),
                        transform: *trans,
                        ..default()
                    },
                ))
                .id();
        }
        if let Ok(mut sprite) = sprites.get_mut(spawner.summon_occluder) {
            let Color::Hsla(hsla) = &mut sprite.color else {
                panic!()
            };
            hsla.alpha -= time.delta_seconds() * 1.5;
        }
        if step.contains(&0.) {
            commands.entity(entity).despawn();
            commands.entity(spawner.enemy).insert((
                Floater { movement_timer: 0. },
                Enemy {
                    health: 3.,
                    size: FLOATER_SIZE,
                },
                HurtIndicator {
                    last_hit: f32::INFINITY,
                    occluder: spawner.summon_occluder,
                },
            ));
        }
    }
}

pub fn floaters(
    mut commands: Commands,
    mut floaters: Query<(Entity, &mut Vel, &mut Floater, &Enemy)>,
    mut transform: Query<&mut Transform, (With<Floater>, Without<Player>)>,
    player: Query<&Transform, With<Player>>,
    tiles: Res<Tiles>,
    time: Res<Time>,
    handles: Res<Handles>,
) {
    const PROPULSION: f32 = 70.;
    for (entity, mut vel, mut floater, enemy) in &mut floaters {
        let mut trans = transform.get_mut(entity).unwrap();
        if floater.movement_timer == 0. {
            let mut dir = Dir2::from_rng(&mut thread_rng()).as_vec2();
            if (trans.translation.y + dir.y * 24. < 10.) & (dir.y < 0.) {
                dir.y *= -1.
            }
            vel.0 = dir;
        }
        vel.0 *= 1. - time.delta_seconds() * 1.0;
        if (floater.movement_timer < 0.8) & (vel.length() != 0.) {
            let dir = vel.normalize();
            vel.0 += dir * time.delta_seconds() * PROPULSION;
        }
        floater.movement_timer += time.delta_seconds();
        let movement = vel.0 * time.delta_seconds();
        let movement = grid_collision(&tiles, trans.translation.xy(), FLOATER_SIZE, movement, true);
        trans.translation += movement.extend(0.);
        let pos = trans.translation.xy();
        vel.0 = movement / time.delta_seconds();
        for other in &transform {
            let other = other.translation.xy();
            if other != pos {
                let distance = other.distance(pos);
                let direction = (pos - other).normalize();
                vel.0 += direction * (2. * FLOATER_SIZE - distance).max(0.) / 2.4;
            }
        }

        if (floater.movement_timer - time.delta_seconds()..floater.movement_timer).contains(&3.)
            & thread_rng().gen_bool(0.5)
        {
            let dir = (player.single().translation.xy() - pos).normalize();
            commands
                .spawn(())
                .with_children(|b| {
                    b.spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::hsv(57., 0.78, 1.),
                            ..default()
                        },
                        texture: handles.bullet.clone(),
                        transform: Transform {
                            translation: vec3(0., 9., 0.),
                            rotation: Quat::from_rotation_z(dir.to_angle()),
                            ..default()
                        },
                        ..default()
                    });
                })
                .insert((
                    Layer(0.0),
                    Transform::from_translation((pos + dir * 5.).extend(0.)),
                    Clearable,
                    Vel(dir * 80.),
                    Bullet { friendly: false },
                    GlobalTransform::default(),
                    InheritedVisibility::default(),
                ));
        }

        if floater.movement_timer > 4. {
            floater.movement_timer = 0.
        }

        if enemy.health <= 0. {
            commands.entity(entity).despawn_recursive();
            commands.spawn(AudioBundle {
                source: handles.sfx_enemy_death.clone(),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: bevy::audio::Volume::new(0.4),
                    ..default()
                },
            });
        }
    }
}

pub fn hurt_indicator(
    mut query: Query<&mut HurtIndicator>,
    mut sprites: Query<&mut Sprite>,
    time: Res<Time>,
) {
    for mut hurt in &mut query {
        let mut sprite = sprites.get_mut(hurt.occluder).unwrap();
        sprite.color = Color::srgba(1., 1., 1., (2. - 8. * hurt.last_hit).clamp(0., 1.));
        hurt.last_hit += time.delta_seconds();
    }
}
