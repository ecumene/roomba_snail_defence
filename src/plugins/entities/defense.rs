use crate::plugins::entities::effects::spawn_flash;
use bevy::prelude::*;

use crate::plugins::entities::enemies::Enemy;

pub struct DefensePlugin;

#[derive(Component)]
pub struct Turret {
    pub target: Option<Entity>,
    pub fire_cooldown: f32,
}

fn shoot_targets(
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut turrets_query: Query<(Entity, &mut Turret, &Transform)>,
    mut enemies_query: Query<(Entity, &mut Enemy, &Transform)>,
) {
    for (turret_entity, mut turret, turret_transform) in turrets_query.iter_mut() {
        if turret.fire_cooldown > 0.0 {
            turret.fire_cooldown -= time.delta_seconds();
            continue;
        }
        let mut found_target = false;
        for (enemy_entity, mut enemy, enemy_transform) in enemies_query.iter_mut() {
            if let Some(dest_turret) = &turret.target {
                if enemy_entity == *dest_turret {
                    found_target = true;
                    if turret.fire_cooldown <= 0.0 {
                        enemy.health -= 1;
                        let mut transform = turret_transform.clone();
                        transform.translation.z += 2.0;
                        spawn_flash(&mut commands, &asset_server, &transform, "shot.png");
                        let mut transform = enemy_transform.clone();
                        transform.translation.z += 2.0;
                        spawn_flash(&mut commands, &asset_server, &transform, "boom.png");
                        turret.fire_cooldown = 1.0;
                    }

                    if enemy.health <= 0 {
                        turret.target = None;
                    }
                }
            }
        }
        if turret.target.is_some() {
            if !found_target {
                turret.target = None;
            }
        }
    }
}

fn discover_targets(
    mut commands: Commands,
    mut turrets_query: Query<(Entity, &mut Turret, &Transform)>,
    mut enemies_query: Query<(Entity, &Enemy, &Transform)>,
) {
    for (turret_entity, mut turret, turret_transform) in turrets_query.iter_mut() {
        for (enemies_entity, enemy, enemy_transform) in enemies_query.iter() {
            if turret_transform
                .translation
                .distance(enemy_transform.translation)
                < 80.0
                && turret.target.is_none()
            {
                turret.target = Some(enemies_entity);
            }
        }
    }
}

impl Plugin for DefensePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(shoot_targets);
        app.add_system(discover_targets);
    }
}
