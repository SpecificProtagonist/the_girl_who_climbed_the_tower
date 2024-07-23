#![allow(clippy::too_many_arguments, clippy::type_complexity)]
mod aseprite;
mod collision;
mod player;

use aseprite::AsepriteLoader;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::sprite::Anchor;
use bevy::{asset::AssetMetaCheck, math::vec2};
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use player::{move_bullets, player_movement, player_shoot, Player};
use rand::{thread_rng, Rng};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            LdtkPlugin,
        ))
        .init_state::<LoadState>()
        .add_loading_state(
            LoadingState::new(LoadState::AssetLoading)
                .continue_to_state(LoadState::Loaded)
                .load_collection::<Handles>(),
        )
        .register_asset_loader(AsepriteLoader)
        .insert_resource(ClearColor(Color::BLACK))
        .register_ldtk_entity::<DoorBundle>("Exit")
        .add_systems(OnEnter(LoadState::Loaded), (setup, spawn_level))
        .add_systems(
            Update,
            (player_movement, player_shoot, move_bullets)
                .chain()
                .run_if(in_state(LoadState::Loaded)),
        )
        .add_systems(Update, play_music)
        .run();
}

fn default<T: Default>() -> T {
    Default::default()
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum LoadState {
    #[default]
    AssetLoading,
    Loaded,
}

#[derive(AssetCollection, Resource)]
struct Handles {
    #[asset(path = "test.aseprite")]
    test: Handle<Image>,
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
    #[asset(path = "enemy.aseprite")]
    _enemy: Handle<Image>,
    #[asset(path = "levels.ldtk")]
    ldtk_project: Handle<LdtkProject>,
}

#[derive(Component, Deref, DerefMut, Copy, Clone, Default, Debug)]
struct Vel(Vec2);

#[derive(Default, Component)]
struct Door;

#[derive(Default, Bundle, LdtkEntity)]
struct DoorBundle {
    door: Door,
    #[sprite_sheet_bundle]
    sprite_bundle: LdtkSpriteSheetBundle,
}

const LAYER_MOB: f32 = 0.;

#[derive(Component)]
struct Music;
fn play_music(mut commands: Commands, query: Query<&Music>, asset_server: Res<AssetServer>) {
    if query.is_empty() {
        commands.spawn((
            Music,
            AudioBundle {
                source: asset_server
                    .load(format!("music/track_{}.ogg", thread_rng().gen_range(1..=7))),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: bevy::audio::Volume::new(0.3),
                    ..default()
                },
            },
        ));
    }
}

fn setup(mut commands: Commands, handles: Res<Handles>) {
    let mut camera = Camera2dBundle {
        transform: Transform::from_xyz(101., 101., 10.),
        ..default()
    };
    camera.projection.scaling_mode = ScalingMode::FixedVertical(176.0);
    commands.spawn(camera);
    commands.spawn((
        Player::default(),
        Vel::default(),
        SpriteBundle {
            transform: Transform::from_xyz(101., 101., LAYER_MOB),
            sprite: Sprite {
                anchor: Anchor::Custom(vec2(0., -0.5 + 3. / 18.)),
                ..default()
            },
            texture: handles.player_down[0].clone(),
            ..default()
        },
    ));

    commands.spawn(SpriteBundle {
        transform: Transform::from_xyz(10. * CELL_SIZE, 10. * CELL_SIZE, 1.),
        texture: handles.test.clone(),
        ..Default::default()
    });
}

static CELL_SIZE: f32 = 12.;
static LEVEL_WIDTH: i32 = 12 * CELL_SIZE as i32;
static LEVEL_HEIGHT: i32 = 12 * CELL_SIZE as i32;

#[derive(Resource)]
struct Level {
    grid: Vec<i32>,
    width: i32,
    height: i32,
}

fn spawn_level(
    mut commands: Commands,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
    handles: Res<Handles>,
) {
    let level_index = 0;
    let level_difficulty = 0;
    let ldtk_project = ldtk_project_assets.get(&handles.ldtk_project).unwrap();
    let ldtk_level = ldtk_project
        .json_data()
        .levels
        .iter()
        .find(|level| {
            (level.world_x == level_index * LEVEL_WIDTH)
                && (level.world_y == level_difficulty * LEVEL_HEIGHT)
        })
        .unwrap();
    let layer = ldtk_level
        .layer_instances
        .as_ref()
        .unwrap()
        .iter()
        .find(|l| l.identifier == "Tiles")
        .unwrap();

    commands.insert_resource(Level {
        grid: layer.int_grid_csv.clone(),
        width: layer.c_wid,
        height: layer.c_hei,
    });
    commands.insert_resource(LevelSelection::iid(ldtk_level.iid.clone()));
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: handles.ldtk_project.clone(),
        transform: Transform::from_xyz(0., 0., -3.),
        ..Default::default()
    });
}
// let width = layer.c_wid;
// let height = layer.c_hei;
// for y in 0..height {
//     for x in 0..width {
//         commands.spawn(SpriteBundle {
//             transform: Transform::from_translation(vec3(
//                 (x as f32 - width as f32 / 2.) * CELL_SIZE,
//                 (y as f32 - height as f32 / 2.) * -CELL_SIZE,
//                 10.,
//             )),
//             texture: if layer.int_grid_csv[(x + y * width) as usize] == 1 {
//                 asset_server.load("enemy.aseprite")
//             } else {
//                 asset_server.load("bullet.aseprite")
//             },
//             ..default()
//         });
//     }
// }
