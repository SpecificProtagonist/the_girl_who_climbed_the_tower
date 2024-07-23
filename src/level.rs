use bevy::{
    math::{ivec2, vec2, vec3},
    prelude::*,
    sprite::Anchor,
    utils::HashMap,
};
use bevy_ecs_ldtk::{assets::LdtkProject, EntityInstance};

use crate::{collision::CollisionGrid, Door, Handles, Layer};

pub static CELL_SIZE: f32 = 12.;
static LEVEL_WIDTH: i32 = 12 * CELL_SIZE as i32;
static LEVEL_HEIGHT: i32 = 12 * CELL_SIZE as i32;

#[derive(Clone, Copy)]
enum ZLayer {
    Subfloor,
    Floor,
    Wall,
    Top,
}

pub fn spawn_level(
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

    // Collision data
    let tile_layer = ldtk_level
        .layer_instances
        .as_ref()
        .unwrap()
        .iter()
        .find(|l| l.identifier == "Tiles")
        .unwrap();

    commands.insert_resource(CollisionGrid {
        grid: tile_layer.int_grid_csv.clone(),
    });
    // commands.insert_resource(LevelSelection::iid(ldtk_level.iid.clone()));
    // commands.spawn(LdtkWorldBundle {
    //     ldtk_handle: handles.ldtk_project.clone(),
    //     transform: Transform::from_xyz(0., 0., -3.),
    //     ..Default::default()
    // });

    assert_eq!((tile_layer.c_wid, tile_layer.c_hei), (16, 16));
    assert_eq!(
        (tile_layer.px_total_offset_x, tile_layer.px_total_offset_y),
        (0, 0)
    );

    // Visuals
    let tileset = ldtk_project
        .json_data()
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
        let z = match z_layers[tile.t as usize] {
            ZLayer::Subfloor => -2.,
            ZLayer::Floor => -1.,
            ZLayer::Wall => 0.,
            ZLayer::Top => 1.,
        } - *count as f32 / 10000.;
        if let Some(&id) = ids.get(&pos.as_ivec2()) {
            if !transparent[id as usize] {
                continue;
            }
        }
        ids.insert(pos.as_ivec2(), tile.t);

        commands.spawn((
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
                Layer(0.),
                SpriteBundle {
                    transform: Transform::from_translation(px_to_world(entity, true).extend(0.)),
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
