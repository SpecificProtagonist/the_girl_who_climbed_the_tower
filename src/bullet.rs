use bevy::prelude::*;

use crate::{
    collision,
    enemy::Enemy,
    level::Tiles,
    player::{HurtPlayer, PlayerEntity, PLAYER_SIZE},
    Hurtable, Vel,
};

const BULLET_SIZE: f32 = 1.5;
const BULLET_DAMAGE: f32 = 1.;

#[derive(Component)]
pub struct Bullet {
    pub friendly: bool,
}

pub fn move_bullets(
    mut commands: Commands,
    tiles: Res<Tiles>,
    mut bullets: Query<(Entity, &mut Transform, &Vel, &Bullet)>,
    mut enemies: Query<(&Transform, &mut Enemy, &mut Hurtable), Without<Bullet>>,
    player: Query<&Transform, (With<PlayerEntity>, Without<Bullet>)>,
    time: Res<Time>,
) {
    for (entity, mut trans, vel, bullet) in &mut bullets {
        let pos = trans.translation.xy();
        let movement = vel.0 * time.delta_seconds();
        if bullet.friendly {
            for (enemy_pos, mut enemy, mut hurt) in &mut enemies {
                if collision::with_ball(
                    enemy_pos.translation.xy(),
                    enemy.size,
                    trans.translation.xy(),
                    BULLET_SIZE,
                    movement,
                ) != movement
                {
                    enemy.health -= BULLET_DAMAGE;
                    hurt.last_hit = 0.;
                    commands.entity(entity).despawn_recursive();
                    break;
                }
            }
        } else if collision::with_ball(
            player.single().translation.xy(),
            PLAYER_SIZE,
            trans.translation.xy(),
            BULLET_SIZE,
            movement,
        ) != movement
        {
            commands.trigger(HurtPlayer);
            commands.entity(entity).despawn_recursive();
        }
        if movement != collision::grid_collision(&tiles, pos, BULLET_SIZE, movement, true) {
            commands.entity(entity).despawn_recursive();
        }
        trans.translation += movement.extend(0.);
    }
}
