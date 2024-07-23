#![allow(clippy::too_many_arguments, clippy::type_complexity)]
mod aseprite;
mod collision;
mod music;
mod player;

use aseprite::AsepriteLoader;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::sprite::Anchor;
use bevy::{asset::AssetMetaCheck, math::vec2};
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use collision::Level;
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

#[derive(Component, Deref, DerefMut, Copy, Clone, Default, Debug)]
struct Vel(Vec2);

#[derive(Default, Component)]
struct Door;

const LAYER_MOB: f32 = 1.;

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
}

static CELL_SIZE: f32 = 12.;
static LEVEL_WIDTH: i32 = 12 * CELL_SIZE as i32;
static LEVEL_HEIGHT: i32 = 12 * CELL_SIZE as i32;

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
    let tile_layer = ldtk_level
        .layer_instances
        .as_ref()
        .unwrap()
        .iter()
        .find(|l| l.identifier == "Tiles")
        .unwrap();

    commands.insert_resource(Level {
        grid: tile_layer.int_grid_csv.clone(),
    });
    commands.insert_resource(LevelSelection::iid(ldtk_level.iid.clone()));
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: handles.ldtk_project.clone(),
        transform: Transform::from_xyz(0., 0., -3.),
        ..Default::default()
    });

    assert_eq!((tile_layer.c_wid, tile_layer.c_hei), (16, 16));
    assert_eq!(
        (tile_layer.px_total_offset_x, tile_layer.px_total_offset_y),
        (0, 0)
    );

    let entity_layer = ldtk_level
        .layer_instances
        .as_ref()
        .unwrap()
        .iter()
        .find(|l| l.identifier == "Entities")
        .unwrap();

    let px_to_world = |entity: &EntityInstance, vertical: bool| {
        let mut y = 192. - entity.px.y as f32;
        if vertical {
            y -= entity.height as f32 / 2.
        }
        vec2(entity.px.x as f32, y)
    };
    for entity in entity_layer
        .entity_instances
        .iter()
        .filter(|e| e.identifier == "Door")
    {
        commands
            .spawn((
                Door,
                SpriteBundle {
                    transform: Transform::from_translation(px_to_world(entity, true).extend(-0.5)),
                    texture: handles.door.clone(),
                    sprite: Sprite {
                        anchor: Anchor::BottomCenter,
                        ..default()
                    },
                    ..default()
                },
            ))
            .with_children(|b| {
                b.spawn(SpriteBundle {
                    transform: Transform::from_xyz(0., 0., 0.1),
                    texture: handles.grate.clone(),
                    sprite: Sprite {
                        anchor: Anchor::BottomCenter,
                        ..default()
                    },
                    ..default()
                });
            });
    }
}
