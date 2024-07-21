#![allow(clippy::too_many_arguments, clippy::type_complexity)]
mod aseprite;

use aseprite::AsepriteLoader;
use bevy::asset::AssetMetaCheck;
use bevy::ecs::component::{ComponentHooks, ComponentId, StorageType};
use bevy::ecs::world::DeferredWorld;
use bevy::math::vec2;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::sprite::Anchor;
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;

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
        .add_systems(PostUpdate, sync_transform)
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
    #[asset(path = "level.ldtk")]
    ldtk_project: Handle<LdtkProject>,
}

#[derive(Deref, DerefMut, Copy, Clone, Debug)]
struct Pos(Vec2);

impl Component for Pos {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(
            |mut world: DeferredWorld, entity: Entity, _id: ComponentId| {
                if !world.entity(entity).contains::<Transform>() {
                    world.commands().entity(entity).insert(Transform::default());
                }
                if !world.entity(entity).contains::<GlobalTransform>() {
                    world
                        .commands()
                        .entity(entity)
                        .insert(GlobalTransform::default());
                }
            },
        );
    }
}

#[derive(Component, Deref, DerefMut, Copy, Clone, Default, Debug)]
struct Vel(Vec2);

fn sync_transform(mut query: Query<(&Pos, &mut Transform)>) {
    for (pos, mut trans) in &mut query {
        trans.translation = pos.extend(0.);
    }
}

#[derive(Component, Default)]
struct Player {
    walk_ani: f32,
    shoot_cooldown: f32,
}

#[derive(Component)]
struct Bullet {
    _sprite: Entity,
}

fn setup(mut commands: Commands, handles: Res<Handles>) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::FixedVertical(176.0);
    commands.spawn(camera);

    commands.spawn((
        Player::default(),
        Pos(Vec2::ZERO),
        Vel::default(),
        SpriteBundle {
            sprite: Sprite {
                anchor: Anchor::BottomCenter,
                ..default()
            },
            texture: handles.player_down[0].clone(),
            ..default()
        },
    ));
    // .with_children(|b| {
    //     b.spawn(camera);
    // });
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player: Query<(
        &mut Pos,
        &mut Vel,
        &mut Player,
        &mut Sprite,
        &mut Handle<Image>,
    )>,
    tex_ass: Res<Handles>,
) {
    let Ok((mut pos, mut velocity, mut player, mut sprite, mut tex)) = player.get_single_mut()
    else {
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
    let speed = 60.;
    let vel = dir.normalize_or_zero() * speed;
    velocity.0 = vel;
    pos.0 += vel * time.delta_seconds();

    if dir != Vec2::ZERO {
        player.walk_ani += time.delta_seconds();
        if player.walk_ani > 0.6 {
            player.walk_ani -= 0.6;
        }
    }
    let index = if player.walk_ani < 0.3 { 0 } else { 1 };
    if dir.y < 0. {
        *tex = tex_ass.player_down[index].clone();
    } else if dir.y > 0. {
        *tex = tex_ass.player_up[index].clone();
    } else if dir.x < 0. {
        *tex = tex_ass.player_side[index].clone();
        sprite.flip_x = true;
    } else if dir.x > 0. {
        *tex = tex_ass.player_side[index].clone();
        sprite.flip_x = false;
    }
}

fn player_shoot(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player: Query<(&Pos, &Vel, &mut Player)>,
    tex_ass: Res<Handles>,
) {
    let Ok((pos, player_vel, mut player)) = player.get_single_mut() else {
        return;
    };
    player.shoot_cooldown -= time.delta_seconds();
    if player.shoot_cooldown > 0. {
        return;
    }

    let mut dir = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        dir -= Vec2::X;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        dir += Vec2::X;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        dir -= Vec2::Y;
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        dir += Vec2::Y;
    }

    if dir == Vec2::ZERO {
        return;
    }
    let dir = dir.normalize();
    let vel = Dir2::new(dir + player_vel.0 * 0.007).unwrap() * 180.;

    player.shoot_cooldown = 0.4;
    let mut sprite = Entity::PLACEHOLDER;
    commands
        .spawn(())
        .with_children(|b| {
            sprite = b
                .spawn((
                    Pos(vec2(0., 12.)),
                    SpriteBundle {
                        texture: tex_ass.bullet.clone(),
                        transform: Transform::from_rotation(Quat::from_rotation_z(dir.to_angle())),
                        ..default()
                    },
                ))
                .id();
        })
        .insert((
            Pos(pos.0 + dir * 5.),
            Vel(vel),
            Bullet { _sprite: sprite },
            GlobalTransform::default(),
            InheritedVisibility::default(),
        ));
}

fn move_bullets(mut bullets: Query<(&mut Pos, &Vel), With<Bullet>>, time: Res<Time>) {
    for (mut pos, vel) in &mut bullets {
        pos.0 += vel.0 * time.delta_seconds();
    }
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
        transform: Transform::from_xyz(-104., -104., -3.),
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
