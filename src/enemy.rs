use bevy::{
    math::{ivec2, vec2, vec3},
    prelude::*,
};
use rand::prelude::*;

use crate::{
    aseprite::Animation,
    bullet::Bullet,
    collision::grid_collision,
    level::{Tile, Tiles, CELL_SIZE},
    player::{HurtPlayer, PlayerEntity, PLAYER_SIZE},
    Clearable, Cycle, Handles, Hurtable, Layer, RoomState, Vel,
};

static FLOATER_SIZE: f32 = 5.;

#[derive(Component)]
pub struct Enemy {
    pub health: f32,
    pub size: f32,
}

#[derive(Component)]
pub struct FloaterA {
    movement_timer: f32,
}

#[derive(Component)]
pub struct FloaterB {
    movement_timer: f32,
}

#[derive(Component)]
pub struct Summoner {
    movement_timer: f32,
    summon_timer: f32,
}

#[derive(Clone, Copy)]
pub enum EnemyKind {
    A,
    B,
    Summoner,
}

#[derive(Component)]
pub struct Spawner {
    timer: f32,
    enemy: Entity,
    summon_occluder: Entity,
    kind: EnemyKind,
}

impl Spawner {
    pub fn create(pos: Vec2, kind: EnemyKind, delay: f32) -> impl Bundle {
        (
            Self {
                timer: delay + 1.25,
                enemy: Entity::PLACEHOLDER,
                summon_occluder: Entity::PLACEHOLDER,
                kind,
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
    for i in 0..4 + cycle.cycle - cycle.rooms[cycle.current_room].difficulty / 2 {
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
        let kinds: &[_] = match cycle.cycle {
            0 => &[(EnemyKind::B, 1.)],
            1 => &[(EnemyKind::A, 1.), (EnemyKind::B, 1.)],
            _ => &[
                (EnemyKind::Summoner, 0.5),
                (EnemyKind::A, 1.),
                (EnemyKind::B, 1.),
            ],
        };
        commands.spawn((
            Spawner::create(
                tile_center + offset,
                kinds
                    .choose_weighted(&mut thread_rng(), |item| item.1)
                    .unwrap()
                    .0,
                delay,
            ),
            Clearable,
        ));
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
                            texture: match spawner.kind {
                                EnemyKind::A | EnemyKind::B => handles.floater_occluded.clone(),
                                EnemyKind::Summoner => handles.summoner_occluded.clone(),
                            },
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
                        texture: match spawner.kind {
                            EnemyKind::A => handles.floater_a.clone(),
                            EnemyKind::B => handles.floater_b.clone(),
                            EnemyKind::Summoner => handles.summoner.clone(),
                        },
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
            match spawner.kind {
                EnemyKind::A => {
                    commands.entity(spawner.enemy).insert((
                        FloaterA { movement_timer: 0. },
                        Enemy {
                            health: 3.,
                            size: FLOATER_SIZE,
                        },
                        Hurtable {
                            last_hit: f32::INFINITY,
                            indicator: spawner.summon_occluder,
                        },
                    ));
                }
                EnemyKind::B => {
                    commands.entity(spawner.enemy).insert((
                        FloaterB { movement_timer: 0. },
                        Enemy {
                            health: 3.,
                            size: FLOATER_SIZE,
                        },
                        Hurtable {
                            last_hit: f32::INFINITY,
                            indicator: spawner.summon_occluder,
                        },
                    ));
                }
                EnemyKind::Summoner => {
                    commands.entity(spawner.enemy).insert((
                        Summoner {
                            movement_timer: 0.,
                            summon_timer: 9.,
                        },
                        Enemy {
                            health: 3.,
                            size: FLOATER_SIZE,
                        },
                        Hurtable {
                            last_hit: f32::INFINITY,
                            indicator: spawner.summon_occluder,
                        },
                    ));
                }
            }
        }
    }
}

pub fn floater_a(
    mut commands: Commands,
    mut floaters: Query<(Entity, &mut Vel, &mut FloaterA, &Enemy)>,
    mut transform: Query<&mut Transform, (With<Enemy>, Without<PlayerEntity>)>,
    player: Query<&Transform, With<PlayerEntity>>,
    tiles: Res<Tiles>,
    time: Res<Time>,
    handles: Res<Handles>,
) {
    const PROPULSION: f32 = 70.;
    for (entity, mut vel, mut floater, enemy) in &mut floaters {
        let mut trans = transform.get_mut(entity).unwrap();
        if floater.movement_timer == 0. {
            let mut dir = Dir2::from_rng(&mut thread_rng()).as_vec2();
            if ((trans.translation.y + dir.y * 24. < 10.) & (dir.y < 0.))
                | ((trans.translation.y + dir.y * 24. > 180.) & (dir.y > 0.))
            {
                dir.y *= -1.
            }
            if ((trans.translation.x + dir.x * 24. < 10.) & (dir.x < 0.))
                | ((trans.translation.x + dir.x * 24. > 180.) & (dir.x > 0.))
            {
                dir.x *= -1.
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

        let player_pos = player.single();
        if player_pos.translation.xy().distance(pos) < FLOATER_SIZE + PLAYER_SIZE {
            commands.trigger(HurtPlayer);
        }

        if (floater.movement_timer - time.delta_seconds()..floater.movement_timer).contains(&3.)
            & thread_rng().gen_bool(0.7)
        {
            let dir = (player_pos.translation.xy() - pos).normalize();
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

pub fn floater_b(
    mut commands: Commands,
    mut floaters: Query<(Entity, &mut Vel, &mut FloaterB, &Enemy)>,
    mut transform: Query<&mut Transform, (With<Enemy>, Without<PlayerEntity>)>,
    player: Query<&Transform, With<PlayerEntity>>,
    tiles: Res<Tiles>,
    time: Res<Time>,
    handles: Res<Handles>,
) {
    const PROPULSION: f32 = 80.;
    let player_pos = player.single();
    for (entity, mut vel, mut floater, enemy) in &mut floaters {
        let mut trans = transform.get_mut(entity).unwrap();
        if floater.movement_timer == 0. {
            let random_dir = Dir2::from_rng(&mut thread_rng()).as_vec2();
            let player_dir =
                (player_pos.translation.xy() - trans.translation.xy()).normalize_or_zero();
            let mut dir = (random_dir + player_dir * 3.).normalize_or_zero();

            if ((trans.translation.y + dir.y * 24. < 10.) & (dir.y < 0.))
                | ((trans.translation.y + dir.y * 24. > 180.) & (dir.y > 0.))
            {
                dir.y *= -1.
            }
            if ((trans.translation.x + dir.x * 24. < 10.) & (dir.x < 0.))
                | ((trans.translation.x + dir.x * 24. > 180.) & (dir.x > 0.))
            {
                dir.x *= -1.
            }
            vel.0 = dir;
        }
        vel.0 *= 1. - time.delta_seconds() * 1.0;
        if (floater.movement_timer < 0.6) & (vel.length() != 0.) {
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

        if player_pos.translation.xy().distance(pos) < FLOATER_SIZE + PLAYER_SIZE {
            commands.trigger(HurtPlayer);
        }

        if floater.movement_timer > 1.5 {
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

pub fn summoner(
    mut commands: Commands,
    mut summoners: Query<(Entity, &mut Vel, &mut Summoner, &Enemy)>,
    mut transform: Query<&mut Transform, (With<Enemy>, Without<PlayerEntity>)>,
    player: Query<&Transform, With<PlayerEntity>>,
    tiles: Res<Tiles>,
    time: Res<Time>,
    handles: Res<Handles>,
    state: Res<State<RoomState>>,
) {
    const PROPULSION: f32 = 70.;
    let player_pos = player.single();
    for (entity, mut vel, mut summoner, enemy) in &mut summoners {
        let mut trans = transform.get_mut(entity).unwrap();

        summoner.summon_timer -= time.delta_seconds();
        if (summoner.summon_timer < 0.) & (*state == RoomState::Fighting) {
            commands.spawn((
                Spawner::create(trans.translation.xy(), EnemyKind::B, 0.),
                Clearable,
            ));
            summoner.summon_timer = 6.
        }

        if summoner.movement_timer == 0. {
            let mut dir = Dir2::from_rng(&mut thread_rng()).as_vec2();
            if ((trans.translation.y + dir.y * 24. < 10.) & (dir.y < 0.))
                | ((trans.translation.y + dir.y * 24. > 180.) & (dir.y > 0.))
            {
                dir.y *= -1.
            }
            if ((trans.translation.x + dir.x * 24. < 10.) & (dir.x < 0.))
                | ((trans.translation.x + dir.x * 24. > 180.) & (dir.x > 0.))
            {
                dir.x *= -1.
            }
            vel.0 = dir;
        }
        vel.0 *= 1. - time.delta_seconds() * 1.0;
        if (summoner.movement_timer < 0.6) & (vel.length() != 0.) {
            let dir = vel.normalize();
            vel.0 += dir * time.delta_seconds() * PROPULSION;
        }
        summoner.movement_timer += time.delta_seconds();
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

        if player_pos.translation.xy().distance(pos) < FLOATER_SIZE + PLAYER_SIZE {
            commands.trigger(HurtPlayer);
        }

        if summoner.movement_timer > 1.5 {
            summoner.movement_timer = 0.
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

// fn enemies(
//     mut enemies: Query<(Entity, &mut Vel), With<Enemy>>,
//     trans: Query<(&Transform, Enemy)>
// ) {
//     for (entity, mut vel) in &mut enemies {
//         let pos = trans.get(entity).unwrap().translation.xy();
//         for (other_pos, other) in &transform {
//             let other = other.translation.xy();
//             if other != pos {
//                 let distance = other.distance(pos);
//                 let direction = (pos - other).normalize();
//                 vel.0 += direction * (2. * FLOATER_SIZE - distance).max(0.) / 2.4;
//             }
//         }
//     }
// }
