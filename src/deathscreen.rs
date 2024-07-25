use bevy::{math::vec3, prelude::*};

use crate::{ldtk::LdtkProject, player::Player, Clearable, Cycle, Handles, RoomState};

#[derive(Resource)]
pub struct DeathTimer(f32);

#[derive(Component)]
pub struct Background;

#[derive(Component)]
pub struct RespawnHint;

#[derive(Component)]
pub struct DespawnOnRespawn;

pub fn death_screen(
    mut commands: Commands,
    time: Res<Time>,
    timer: Option<ResMut<DeathTimer>>,
    mut next_roomstate: ResMut<NextState<RoomState>>,
    handles: Res<Handles>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut background: Query<&mut Sprite, With<Background>>,
    despawn: Query<Entity, With<DespawnOnRespawn>>,
    cycle: Res<Cycle>,
    ldtk: Res<LdtkProject>,
    clearable: Query<Entity, With<Clearable>>,
    mut respawn_hint: Query<&mut Sprite, (With<RespawnHint>, Without<Background>)>,
) {
    let Some(mut timer) = timer else {
        commands.insert_resource(DeathTimer(0.));
        commands.spawn((
            Background,
            DespawnOnRespawn,
            SpriteBundle {
                texture: handles.black.clone(),
                transform: Transform {
                    translation: vec3(0., 0., 10.),
                    scale: vec3(400., 400., 1.),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::srgba(0., 0., 0., 0.),
                    ..default()
                },
                ..default()
            },
        ));
        return;
    };

    timer.0 += time.delta_seconds();

    let Sprite {
        color: Color::Srgba(ref mut background),
        ..
    } = &mut *background.single_mut()
    else {
        panic!()
    };
    background.alpha = (timer.0 * 0.2).powf(0.3).min(1.);

    if (timer.0 - time.delta_seconds()..timer.0).contains(&3.) {
        commands.spawn((
            DespawnOnRespawn,
            SpriteBundle {
                texture: handles.ouroboros.clone(),
                transform: Transform {
                    translation: vec3(101., 101., 11.),
                    scale: vec3(2., 2., 1.),
                    ..default()
                },
                ..default()
            },
        ));
        commands.spawn((
            DespawnOnRespawn,
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: format!("{}", cycle.cycle),
                        style: TextStyle {
                            font: handles.font_score.clone(),
                            font_size: 16.,
                            color: Color::WHITE,
                        },
                    }],
                    ..default()
                },
                transform: Transform {
                    translation: vec3(101., 101., 11.),
                    scale: vec3(2., 2., 1.),
                    ..default()
                },
                ..default()
            },
        ));
    }

    if (timer.0 - time.delta_seconds()..timer.0).contains(&10.) {
        commands.spawn((
            DespawnOnRespawn,
            RespawnHint,
            SpriteBundle {
                texture: handles.key_enter.clone(),
                transform: Transform::from_xyz(101., 40., 11.),
                sprite: Sprite {
                    color: Color::srgba(0., 0., 0., 0.),
                    ..default()
                },
                ..default()
            },
        ));
    }

    if let Ok(mut sprite) = respawn_hint.get_single_mut() {
        sprite.color = Color::srgba(1., 1., 1., time.elapsed_seconds().sin() * 0.5 + 0.5);
    }

    if (timer.0 > 3.) & keyboard_input.just_pressed(KeyCode::Enter) {
        for entity in &despawn {
            commands.entity(entity).despawn();
        }
        commands.remove_resource::<DeathTimer>();
        // Reset
        commands.insert_resource(Cycle::new(&ldtk));
        commands.insert_resource(Player::default());
        // Clear room
        for entity in &clearable {
            commands.entity(entity).despawn_recursive()
        }
        next_roomstate.set(RoomState::Fighting);
    }
}
