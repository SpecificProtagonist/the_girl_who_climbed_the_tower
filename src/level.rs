use std::f32::consts::PI;

use bevy::{
    math::{vec2, vec3},
    prelude::*,
    sprite::Anchor,
    utils::HashMap,
};

use crate::{
    bullet::Bullet,
    ldtk::{EntityInstance, LdtkProject},
    player::{PlayerEntity, PlayerHurtFlash},
    Clearable, Cycle, Door, Gargoyle, Handles, Layer, Vel,
};

pub static CELL_SIZE: f32 = 12.;
static LEVEL_WIDTH: i32 = 16 * CELL_SIZE as i32;
static LEVEL_HEIGHT: i32 = 16 * CELL_SIZE as i32;

#[derive(Clone, Copy)]
enum ZLayer {
    Subfloor,
    Floor,
    Wall,
    Top,
}

impl ZLayer {
    fn base_z(self) -> f32 {
        match self {
            ZLayer::Subfloor => -2.,
            ZLayer::Floor => -1.,
            ZLayer::Wall => 0.,
            ZLayer::Top => 1.,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd)]
pub enum Tile {
    Wall,
    Floor,
    Pit,
}

#[derive(Resource)]
pub struct Tiles {
    pub grid: Vec<Tile>,
}

impl std::ops::Index<IVec2> for Tiles {
    type Output = Tile;

    fn index(&self, index: IVec2) -> &Self::Output {
        &self.grid[(index.x + (15 - index.y) * 16) as usize]
    }
}

#[derive(Default, Component)]
pub struct DeleteOnClear;

pub fn spawn_level(
    mut commands: Commands,
    ldtk: Res<LdtkProject>,
    handles: Res<Handles>,
    cycle: Res<Cycle>,
) {
    let cycle_progress = cycle.current_room as f32 / cycle.rooms.len() as f32 + 0.1;
    let ldtk_level = ldtk
        .levels
        .iter()
        .find(|level| {
            (level.world_x == cycle.rooms[cycle.current_room].id * LEVEL_WIDTH)
                && (level.world_y == cycle.rooms[cycle.current_room].difficulty * LEVEL_HEIGHT)
        })
        .unwrap();

    // Collision data
    let tile_layer = ldtk_level
        .layer_instances
        .as_ref()
        .unwrap()
        .iter()
        .find(|l| l.identifier == "Tiles")
        .unwrap();

    commands.insert_resource(Tiles {
        grid: tile_layer
            .int_grid_csv
            .iter()
            .map(|t| match t {
                1 => Tile::Floor,
                2 => Tile::Pit,
                _ => Tile::Wall,
            })
            .collect(),
    });

    assert_eq!((tile_layer.c_width, tile_layer.c_height), (16, 16));
    assert_eq!(
        (tile_layer.px_total_offset_x, tile_layer.px_total_offset_y),
        (0, 0)
    );

    // Visuals
    let tileset = ldtk
        .defs
        .tilesets
        .iter()
        .find(|t| t.identifier == "Tiles")
        .unwrap();
    let mut z_layers = [ZLayer::Floor; 100];
    let mut transparent = [false; 100];
    for value in &tileset.enum_tags {
        match value.enum_value_id.as_str() {
            "Subfloor" => {
                for index in &value.tile_ids {
                    z_layers[*index as usize] = ZLayer::Subfloor;
                }
            }
            "Floor" => {
                for index in &value.tile_ids {
                    z_layers[*index as usize] = ZLayer::Floor;
                }
            }
            "Wall" => {
                for index in &value.tile_ids {
                    z_layers[*index as usize] = ZLayer::Wall;
                }
            }
            "Top" => {
                for index in &value.tile_ids {
                    z_layers[*index as usize] = ZLayer::Top;
                }
            }
            "Transparent" => {
                for index in &value.tile_ids {
                    transparent[*index as usize] = true;
                }
            }
            _ => panic!(),
        }
    }

    let auto_layer = ldtk_level
        .layer_instances
        .as_ref()
        .unwrap()
        .iter()
        .find(|l| l.identifier == "AutoLayer")
        .unwrap();

    let mut counts = HashMap::new();
    let mut ids = HashMap::new();
    for tile in auto_layer.auto_layer_tiles.iter().rev() {
        let pos = vec2(tile.px.x as f32, CELL_SIZE * 15. - tile.px.y as f32);
        let count = counts.entry(pos.as_ivec2()).or_insert(0);
        *count += 1;
        let z = z_layers[tile.t as usize].base_z() - *count as f32 / 10000.;
        if let Some(&id) = ids.get(&pos.as_ivec2()) {
            if !transparent[id as usize] {
                continue;
            }
        }
        ids.insert(pos.as_ivec2(), tile.t);

        commands.spawn((
            Clearable,
            Layer(z),
            SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::BottomLeft,
                    ..default()
                },
                transform: Transform {
                    translation: vec3(pos.x, pos.y, 0.),
                    // Workaround gaps from not being pixel-perfect
                    scale: vec3(1.01, 1.01, 1.),
                    ..default()
                },
                texture: handles.tiles.clone(),
                ..default()
            },
            TextureAtlas {
                layout: handles.layout.clone(),
                index: tile.t as usize,
            },
        ));
    }

    // Markers
    let entity_layer = ldtk_level
        .layer_instances
        .as_ref()
        .unwrap()
        .iter()
        .find(|l| l.identifier == "Entities")
        .unwrap();

    let px_to_world = |entity: &EntityInstance| vec2(entity.px.x as f32, 192. - entity.px.y as f32);
    for entity in entity_layer
        .entity_instances
        .iter()
        .filter(|e| e.identifier == "Door")
    {
        commands
            .spawn((
                Door,
                Clearable,
                Layer(0.),
                SpriteBundle {
                    transform: Transform::from_translation(
                        px_to_world(entity).extend(0.) - Vec3::Y * 12.,
                    ),
                    texture: handles.door.clone(),
                    sprite: Sprite {
                        anchor: Anchor::BottomCenter,
                        ..default()
                    },
                    ..default()
                },
            ))
            .with_children(|b| {
                b.spawn((
                    DeleteOnClear,
                    SpriteBundle {
                        transform: Transform::from_xyz(0., 0., 0.001),
                        texture: handles.grate.clone(),
                        sprite: Sprite {
                            anchor: Anchor::BottomCenter,
                            ..default()
                        },
                        ..default()
                    },
                ));
                b.spawn((
                    DeleteOnClear,
                    SpriteBundle {
                        transform: Transform {
                            translation: vec3(0., 8., 0.0005),
                            rotation: Quat::from_rotation_z(cycle_progress * PI * -2.),
                            ..default()
                        },
                        texture: handles.cycle_indicator.clone(),
                        ..default()
                    },
                ));
            });
    }

    for entity in entity_layer
        .entity_instances
        .iter()
        .filter(|e| e.identifier == "Gargoyle")
    {
        commands.spawn((
            Gargoyle,
            Clearable,
            Layer(0.),
            SpriteBundle {
                transform: Transform::from_translation(
                    px_to_world(entity).extend(0.) - Vec3::Y * 4.,
                ),
                texture: handles.gargoyle.clone(),
                sprite: Sprite {
                    anchor: Anchor::BottomCenter,
                    ..default()
                },
                ..default()
            },
        ));
    }

    for entity in entity_layer
        .entity_instances
        .iter()
        .filter(|e| e.identifier == "Player")
    {
        commands
            .spawn((
                PlayerEntity,
                Layer(0.0),
                Clearable,
                Vel::default(),
                SpriteBundle {
                    transform: Transform::from_translation(px_to_world(entity).extend(0.)),
                    sprite: Sprite {
                        anchor: Anchor::Custom(vec2(0., -0.5 + 3. / 18.)),
                        ..default()
                    },
                    texture: handles.player_down[0].clone(),
                    ..default()
                },
            ))
            .with_children(|b| {
                b.spawn((
                    PlayerHurtFlash,
                    SpriteBundle {
                        texture: handles.player_hurt.clone(),
                        transform: Transform {
                            translation: vec3(0., 6., -0.001),
                            ..default()
                        },
                        sprite: Sprite {
                            color: Color::srgba(1., 1., 1., 0.),
                            ..default()
                        },
                        ..default()
                    },
                ));
            });
    }
}

pub fn open_door(mut commands: Commands, query: Query<Entity, With<DeleteOnClear>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn deactivate_gargoyles(
    mut query: Query<&mut Handle<Image>, With<Gargoyle>>,
    handles: Res<Handles>,
) {
    for mut tex in &mut query {
        *tex = handles.gargoyle_inactive.clone();
    }
}

pub fn gargoyles(
    mut commands: Commands,
    gargoyles: Query<&Transform, With<Gargoyle>>,
    handles: Res<Handles>,
    mut cooldown: Local<f32>,
    time: Res<Time>,
) {
    *cooldown -= time.delta_seconds();
    if *cooldown < 0. {
        *cooldown = 3.;
        for trans in &gargoyles {
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
                            rotation: Quat::from_rotation_z(0.5 * -PI),
                            ..default()
                        },
                        ..default()
                    });
                })
                .insert((
                    Layer(0.0),
                    *trans,
                    Clearable,
                    Vel(vec2(0., -70.)),
                    Bullet { friendly: false },
                    GlobalTransform::default(),
                    InheritedVisibility::default(),
                ));
        }
    }
}
