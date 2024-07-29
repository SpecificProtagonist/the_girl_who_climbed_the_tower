use bevy::prelude::*;
use rand::prelude::*;

#[derive(Component)]
pub struct Music;

#[derive(Resource)]
pub struct MusicVolume(f32);

impl Default for MusicVolume {
    fn default() -> Self {
        Self(0.2)
    }
}

pub fn play_music(
    mut commands: Commands,
    query: Query<&Music>,
    asset_server: Res<AssetServer>,
    volume: Res<MusicVolume>,
) {
    if query.is_empty() {
        commands.spawn((
            Music,
            AudioBundle {
                source: asset_server
                    .load(format!("music/track_{}.ogg", thread_rng().gen_range(1..=7))),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: bevy::audio::Volume::new(volume.0),
                    ..default()
                },
            },
        ));
    }
}

pub fn music_volume(
    query: Query<&AudioSink, With<Music>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut volume: ResMut<MusicVolume>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        volume.0 = (volume.0
            + if keyboard_input.pressed(KeyCode::ShiftLeft)
                | keyboard_input.pressed(KeyCode::ShiftRight)
            {
                -0.05
            } else {
                0.05
            })
        .clamp(0., 1.);
    }
    let Ok(player) = query.get_single() else {
        return;
    };
    player.set_volume(volume.0);
}
