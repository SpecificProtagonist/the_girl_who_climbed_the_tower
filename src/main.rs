#![allow(clippy::too_many_arguments, clippy::type_complexity)]
mod aseprite;
mod collision;
mod level;
mod music;
mod player;

use aseprite::AsepriteLoader;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::sprite::Anchor;
use bevy::{asset::AssetMetaCheck, math::vec2};
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use collision::CollisionGrid;
use level::spawn_level;
use music::{music_volume, play_music};
use player::{move_bullets, player_movement, player_shoot, Player};

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
        .add_systems(OnEnter(LoadState::Loaded), (setup, spawn_level))
        .add_systems(
            Update,
            (player_movement, player_shoot, move_bullets)
                .chain()
                .run_if(in_state(LoadState::Loaded)),
        )
        .add_systems(Update, (play_music, music_volume))
        .add_systems(PostUpdate, sync_layer)
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
    #[asset(texture_atlas_layout(tile_size_x = 12, tile_size_y = 12, columns = 12, rows = 12))]
    layout: Handle<TextureAtlasLayout>,
    #[asset(path = "tiles.aseprite")]
    tiles: Handle<Image>,
    #[asset(path = "test.aseprite")]
    _test: Handle<Image>,
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
    #[asset(path = "door.aseprite")]
    door: Handle<Image>,
    #[asset(path = "grate.aseprite")]
    grate: Handle<Image>,
    #[asset(path = "levels.ldtk")]
    ldtk_project: Handle<LdtkProject>,
}

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

fn setup(mut commands: Commands, handles: Res<Handles>) {
    let mut camera = Camera2dBundle {
        transform: Transform::from_xyz(101., 101., 10.),
        ..default()
    };
    camera.projection.scaling_mode = ScalingMode::FixedVertical(176.0);
    commands.spawn(camera);
    commands.spawn((
        Player::default(),
        Layer(0.0),
        Vel::default(),
        SpriteBundle {
            transform: Transform::from_xyz(101., 101., 0.),
            sprite: Sprite {
                anchor: Anchor::Custom(vec2(0., -0.5 + 3. / 18.)),
                ..default()
            },
            texture: handles.player_down[0].clone(),
            ..default()
        },
    ));
}
