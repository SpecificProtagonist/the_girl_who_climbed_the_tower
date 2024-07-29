use bevy::{math::vec3, prelude::*};

use crate::{
    bullet::Bullet, collision::grid_collision, level::Tiles, Clearable, Handles, Layer, RoomState,
    Vel,
};

pub const PLAYER_SIZE: f32 = 4.;

#[derive(Component)]
pub struct PlayerHurtFlash;

#[derive(Component)]
pub struct PlayerEntity;

#[derive(Resource)]
pub struct Player {
    walk_ani: f32,
    shoot_cooldown: f32,
    pub health: i32,
    pub invulnerable: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            walk_ani: 0.,
            shoot_cooldown: 0.,
            health: 3,
            invulnerable: 0.,
        }
    }
}

pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_entity: Query<
        (&mut Transform, &mut Vel, &mut Sprite, &mut Handle<Image>),
        With<PlayerEntity>,
    >,
    mut player: ResMut<Player>,
    handles: Res<Handles>,
    tiles: Res<Tiles>,
) {
    if time.delta_seconds() <= 0. {
        // TODO: investigate NaN velocity bug, then remove this
        return;
    }
    let Ok((mut pos, mut velocity, mut sprite, mut tex)) = player_entity.get_single_mut() else {
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
    const PLAYER_SPEED: f32 = 28.;
    let vel = if dir == Vec2::ZERO {
        velocity.0 * (1. - time.delta_seconds() * 15.)
    } else {
        dir.normalize_or_zero() * PLAYER_SPEED + 0.4 * velocity.0
    };
    let attempt_movement = vel * time.delta_seconds();
    let movement = grid_collision(
        &tiles,
        pos.translation.xy(),
        PLAYER_SIZE,
        attempt_movement,
        false,
    );
    if movement.x.is_nan() | movement.y.is_nan() {
        // TODO: investigate NaN velocity bug, then remove this
        return;
    }
    velocity.0 = movement / time.delta_seconds();
    pos.translation += movement.extend(0.);

    if dir != Vec2::ZERO {
        player.walk_ani += time.delta_seconds();
        if player.walk_ani > 0.6 {
            player.walk_ani -= 0.6;
        }
    }
    let index = if player.walk_ani < 0.3 { 0 } else { 1 };
    if dir.y < 0. {
        *tex = handles.player_down[index].clone();
    } else if dir.y > 0. {
        *tex = handles.player_up[index].clone();
    } else if dir.x < 0. {
        *tex = handles.player_side[index].clone();
        sprite.flip_x = true;
    } else if dir.x > 0. {
        *tex = handles.player_side[index].clone();
        sprite.flip_x = false;
    }
}

pub fn player_shoot(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    player_entity: Query<(&Transform, &Vel), With<PlayerEntity>>,
    mut player: ResMut<Player>,
    handles: Res<Handles>,
) {
    let Ok((pos, player_vel)) = player_entity.get_single() else {
        return;
    };
    player.shoot_cooldown -= time.delta_seconds();
    if player.shoot_cooldown > 0. {
        return;
    }

    let mut dir = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::ArrowLeft) | keyboard_input.pressed(KeyCode::KeyJ) {
        dir -= Vec2::X;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) | keyboard_input.pressed(KeyCode::KeyL) {
        dir += Vec2::X;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) | keyboard_input.pressed(KeyCode::KeyK) {
        dir -= Vec2::Y;
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) | keyboard_input.pressed(KeyCode::KeyI) {
        dir += Vec2::Y;
    }

    if dir == Vec2::ZERO {
        return;
    }
    let dir = dir.normalize();
    let vel = Dir2::new(dir + player_vel.0 * 0.005).unwrap() * 180.;

    player.shoot_cooldown = 0.4;
    commands.spawn(AudioBundle {
        source: handles.sfx_shoot.clone(),
        settings: PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: bevy::audio::Volume::new(0.4),
            ..default()
        },
    });
    commands
        .spawn(())
        .with_children(|b| {
            b.spawn(SpriteBundle {
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
            Transform::from_translation(pos.translation + dir.extend(0.) * 5.),
            Clearable,
            Vel(vel),
            Bullet { friendly: true },
            GlobalTransform::default(),
            InheritedVisibility::default(),
        ));
}

#[derive(Event)]
pub struct HurtPlayer;

pub fn player_hurt(
    _: Trigger<HurtPlayer>,
    mut commands: Commands,
    mut player: ResMut<Player>,
    state: Res<State<RoomState>>,
    mut next: ResMut<NextState<RoomState>>,
    handles: Res<Handles>,
) {
    if (player.invulnerable > 0.) | (*state == RoomState::PlayerDead) {
        return;
    }
    player.invulnerable = 1.;
    player.health -= 1;
    commands.spawn(AudioBundle {
        source: handles.sfx_hurt.clone(),
        settings: PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: bevy::audio::Volume::new(0.4),
            ..default()
        },
    });
    if player.health <= 0 {
        next.set(RoomState::PlayerDead);
        commands.spawn(AudioBundle {
            source: handles.sfx_death.clone(),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                volume: bevy::audio::Volume::new(0.2),
                ..default()
            },
        });
    }
}

pub fn player_health(
    mut player: ResMut<Player>,
    mut flash: Query<&mut Sprite, With<PlayerHurtFlash>>,
    mut hearts: Query<(&mut Handle<Image>, &HeartUI), Without<PlayerHurtFlash>>,
    time: Res<Time>,
    handles: Res<Handles>,
) {
    player.invulnerable -= time.delta_seconds();
    flash.single_mut().color =
        Color::srgba(1., 1., 1., (player.invulnerable * 15. - 14.).clamp(0., 1.0));
    for (mut tex, heart) in &mut hearts {
        *tex = if heart.0 <= player.health {
            handles.heart.clone()
        } else {
            handles.heart_empty.clone()
        };
    }
}

#[derive(Component)]
pub struct HeartUI(i32);

pub fn player_hearts_init(mut commands: Commands) {
    for i in 1..=3 {
        commands.spawn((
            HeartUI(i),
            SpriteBundle {
                transform: Transform::from_xyz(-7., 193. - i as f32 * 14., 4.),
                ..default()
            },
        ));
    }
}
