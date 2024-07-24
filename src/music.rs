use bevy::prelude::*;
use rand::prelude::*;

#[derive(Component)]
pub struct Music;

pub fn play_music(mut commands: Commands, query: Query<&Music>, asset_server: Res<AssetServer>) {
    if query.is_empty() {
        commands.spawn((
            Music,
            AudioBundle {
                source: asset_server
                    .load(format!("music/track_{}.ogg", thread_rng().gen_range(1..=7))),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: bevy::audio::Volume::new(0.25),
                    ..default()
                },
            },
        ));
    }
}

pub fn music_volume(
    query: Query<&AudioSink, With<Music>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(player) = query.get_single() else {
        return;
    };
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        player.set_volume(
            (player.volume()
                + if keyboard_input.pressed(KeyCode::ShiftLeft)
                    | keyboard_input.pressed(KeyCode::ShiftRight)
                {
                    -0.05
                } else {
                    0.05
                })
            .clamp(0., 1.),
        );
    }
}
