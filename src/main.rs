#![allow(clippy::too_many_arguments, clippy::type_complexity)]
mod aseprite;
mod collision;
mod deathscreen;
mod enemy;
mod ldtk;
mod level;
mod music;
mod player;

use aseprite::{animations, AnimationData, AsepriteAniLoader, AsepriteImageLoader};
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::utils::HashMap;
use bevy_asset_loader::prelude::*;
use deathscreen::death_screen;
use enemy::{floaters, hurt_indicator, spawn_enemies, spawners, Enemy, Spawner};
use ldtk::{LdtkLoader, LdtkProject};
use level::{deactivate_gargoyles, gargoyles, open_door, spawn_level};
use music::{music_volume, play_music};
use player::{move_bullets, player_movement, player_shoot, Player};
use rand::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(ImagePlugin::default_nearest()),))
        .init_state::<RoomState>()
        .init_state::<LoadState>()
        .add_loading_state(
            LoadingState::new(LoadState::AssetLoading)
                .continue_to_state(LoadState::Loaded)
                .load_collection::<Handles>(),
        )
        .init_asset::<LdtkProject>()
        .register_asset_loader(LdtkLoader)
        .register_asset_loader(AsepriteImageLoader)
        .init_asset::<AnimationData>()
        .register_asset_loader(AsepriteAniLoader)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(OnEnter(LoadState::Loaded), setup)
        .add_systems(
            OnEnter(RoomState::Fighting),
            (spawn_level, spawn_enemies).chain(),
        )
        .add_systems(
            Update,
            (
                (player_movement, player_shoot).run_if(not(in_state(RoomState::PlayerDead))),
                move_bullets,
                spawners,
                floaters,
                gargoyles,
                hurt_indicator,
                check_cleared.run_if(in_state(RoomState::Fighting)),
                check_exit.run_if(in_state(RoomState::Cleared)),
                death_screen.run_if(in_state(RoomState::PlayerDead)),
            )
                .chain()
                .run_if(in_state(LoadState::Loaded))
                .run_if(not(in_state(RoomState::Loading))),
        )
        .add_systems(
            OnEnter(RoomState::Cleared),
            (open_door, deactivate_gargoyles),
        )
        .add_systems(Update, (play_music, music_volume))
        .add_systems(PostUpdate, (sync_layer, animations))
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum RoomState {
    #[default]
    Loading,
    Fighting,
    Cleared,
    PlayerDead,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum LoadState {
    #[default]
    AssetLoading,
    Loaded,
}

#[derive(AssetCollection, Resource)]
struct Handles {
    #[asset(path = "levels.ldtk")]
    ldtk_project: Handle<LdtkProject>,

    #[asset(texture_atlas_layout(tile_size_x = 12, tile_size_y = 12, columns = 12, rows = 12))]
    layout: Handle<TextureAtlasLayout>,
    #[asset(path = "tiles.aseprite")]
    tiles: Handle<Image>,

    #[asset(
        paths("player_down_0.aseprite", "player_down_1.aseprite"),
        collection(typed)
    )]
    player_down: Vec<Handle<Image>>,
    #[asset(
        paths("player_up_0.aseprite", "player_up_1.aseprite"),
        collection(typed)
    )]
    player_up: Vec<Handle<Image>>,
    #[asset(
        paths("player_side_0.aseprite", "player_side_1.aseprite"),
        collection(typed)
    )]
    player_side: Vec<Handle<Image>>,
    #[asset(path = "bullet.aseprite")]
    bullet: Handle<Image>,
    #[asset(path = "gargoyle.aseprite")]
    gargoyle: Handle<Image>,
    #[asset(path = "gargoyle_inactive.aseprite")]
    gargoyle_inactive: Handle<Image>,
    #[asset(path = "enemy.aseprite")]
    enemy: Handle<Image>,
    #[asset(path = "enemy_summon.aseprite")]
    enemy_summon: Handle<Image>,
    #[asset(path = "door.aseprite")]
    door: Handle<Image>,
    #[asset(path = "grate_circle.aseprite")]
    grate: Handle<Image>,
    #[asset(path = "cycle_indicator.aseprite")]
    cycle_indicator: Handle<Image>,
    #[asset(path = "ouroboros.aseprite")]
    ouroboros: Handle<Image>,
    #[asset(path = "black.aseprite")]
    black: Handle<Image>,
    #[asset(path = "key_enter.aseprite")]
    key_enter: Handle<Image>,

    #[asset(path = "summon_ani.aseprite")]
    summon: Handle<AnimationData>,

    #[asset(path = "sfx/enemy_death.ogg")]
    sfx_enemy_death: Handle<AudioSource>,
    #[asset(path = "sfx/summon.ogg")]
    sfx_summon: Handle<AudioSource>,
    #[asset(path = "sfx/shoot.ogg")]
    sfx_shoot: Handle<AudioSource>,

    #[asset(path = "bitmgothic.ttf")]
    font_score: Handle<Font>,
}

#[derive(Component)]
struct Clearable;

#[derive(Component)]
struct Layer(f32);

fn sync_layer(mut query: Query<(&mut Transform, &Layer)>) {
    for (mut transform, layer) in &mut query {
        transform.translation.z = layer.0 - transform.translation.y / 1000.;
    }
}

#[derive(Component, Deref, DerefMut, Copy, Clone, Default, Debug)]
struct Vel(Vec2);

#[derive(Default, Component)]
struct Door;

#[derive(Default, Component)]
struct Gargoyle;

fn setup(
    mut commands: Commands,
    handles: Res<Handles>,
    mut windows: Query<&mut Window>,
    mut ldtk: ResMut<Assets<LdtkProject>>,
    mut next_state: ResMut<NextState<RoomState>>,
) {
    let ldtk = ldtk.remove(handles.ldtk_project.id()).unwrap();
    commands.insert_resource(Cycle::new(&ldtk));
    commands.insert_resource(ldtk);

    windows.single_mut().title = "The girl who climbed the tower".to_owned();

    let mut camera = Camera2dBundle {
        transform: Transform::from_xyz(101., 101., 10.),
        ..default()
    };
    camera.projection.scaling_mode = ScalingMode::FixedVertical(176.0);
    commands.spawn(camera);

    next_state.set(RoomState::Fighting);
}

fn check_cleared(
    mut next_state: ResMut<NextState<RoomState>>,
    query: Query<(), Or<(With<Enemy>, With<Spawner>)>>,
) {
    if query.is_empty() {
        next_state.set(RoomState::Cleared);
    }
}

fn check_exit(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    door: Query<&Transform, With<Door>>,
    clearable: Query<Entity, With<Clearable>>,
    mut cycle: ResMut<Cycle>,
    mut next_state: ResMut<NextState<RoomState>>,
) {
    // Check for exit
    let player = player.single().translation.xy();
    let door = door.single().translation.xy();
    let off = (door - player).abs();
    if (off.x > 5.) | (off.y > 5.) {
        return;
    }

    // Clear room
    for entity in &clearable {
        commands.entity(entity).despawn_recursive()
    }

    // Next room
    cycle.current_room += 1;
    if cycle.current_room == cycle.rooms.len() {
        cycle.current_room = 0;
        cycle.cycle += 1;
        let room = cycle.rooms.choose_mut(&mut thread_rng()).unwrap();
        if room.difficulty < room.max_difficulty {
            room.difficulty += 1;
        }
    }

    next_state.set(RoomState::Fighting);
}

struct Room {
    id: i32,
    difficulty: i32,
    max_difficulty: i32,
}

#[derive(Resource)]
struct Cycle {
    rooms: Vec<Room>,
    current_room: usize,
    cycle: i32,
}

impl Cycle {
    fn new(ldtk: &LdtkProject) -> Self {
        let mut available = HashMap::new();
        for level in &ldtk.levels {
            let id = level.world_x / 192;
            let difficulty = level.world_y / 192;
            let max_difficulty = available.entry(id).or_insert(0);
            *max_difficulty = (*max_difficulty).max(difficulty);
        }
        let mut rooms = Vec::new();
        for (id, max) in available {
            rooms.push(Room {
                id,
                difficulty: 0,
                max_difficulty: max,
            });
        }
        rooms.shuffle(&mut thread_rng());
        Self {
            rooms,
            current_room: 0,
            cycle: 0,
        }
    }
}
