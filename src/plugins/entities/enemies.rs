use crate::plugins::commands::{ClientEvent, EventType};
use crate::Comms;
use crate::Game;
use crate::Server;
use bevy::prelude::*;

const PATH_1: [(f32, f32); 16] = [
    (15.0, 102.0),
    (117.0, 107.0),
    (150.0, 80.0),
    (207.0, 71.0),
    (258.0, 73.0),
    (283.0, 49.0),
    (353.0, 61.0),
    (417.0, 53.0),
    (350.0, 83.0),
    (480.0, 141.0),
    (549.0, 172.0),
    (646.0, 166.0),
    (708.0, 123.0),
    (780.0, 97.0),
    (880.0, 95.0),
    (980.0, 183.0),
];

const PATH_2: [(f32, f32); 16] = [
    (33.0, 240.0),
    (61.0, 230.0),
    (93.0, 192.0),
    (132.0, 174.0),
    (278.0, 153.0),
    (278.0, 153.0),
    (353.0, 133.0),
    (401.0, 142.0),
    (449.0, 120.0),
    (481.0, 53.0),
    (542.0, 27.0),
    (643.0, 49.0),
    (711.0, 74.0),
    (780.0, 96.0),
    (878.0, 94.0),
    (981.0, 188.0),
];

pub struct EnemiesPlugin;

#[derive(Component)]
struct SpawnTimer {
    last_spawned: i8,
    elapsed: f32,
}

#[derive(Component)]
pub struct Enemy {
    pub health: i8,
    target_index: usize,
    path: i8,
}

fn move_enemies(
    mut commands: Commands,
    sender: Res<Comms<Server>>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Enemy, &mut Transform)>,
) {
    for (entity, mut enemy, mut transform) in query.iter_mut() {
        if enemy.health <= 0 {
            commands.entity(entity).despawn();
            let event = ClientEvent {
                event_type: EventType::Killed,
                x: transform.translation.x as u32,
                y: transform.translation.y as u32,
            };
            sender
                .send
                .send(serde_json::to_string(&event).unwrap())
                .unwrap();
            return;
        }

        let path = if enemy.path == 0 { PATH_1 } else { PATH_2 };
        let target = path[enemy.target_index];
        let dx = target.0 - transform.translation.x;
        let dy = target.1 - transform.translation.y;
        let distance = (dx.powi(2) + dy.powi(2)).sqrt();
        let dx = dx / distance;
        let dy = dy / distance;
        transform.translation.x += dx * time.delta_seconds() * 10.0;
        transform.translation.y += dy * time.delta_seconds() * 10.0;
        if distance < 0.1 {
            enemy.target_index += 1;
            if enemy.target_index >= path.len() {
                panic!("YOU LOST! THE SNAIL HAS BEEN ROOMBA'D!")
            }
        }
    }
}

fn spawn_enemies(
    mut query: Query<&mut SpawnTimer>,
    enemies: Query<&mut Enemy>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut commands: Commands,
) {
    if game.timeout <= 0.0 {
        for mut spawner in query.iter_mut() {
            let path;
            spawner.last_spawned = if spawner.last_spawned == 0 {
                path = PATH_2;
                1
            } else {
                path = PATH_1;
                0
            };
            if game.enemies_left != 0 {
                if spawner.elapsed < 0.0 {
                    game.enemies_left -= 1;
                    println!("Enemies left: {}", game.enemies_left);
                    commands
                        .spawn_bundle(SpriteBundle {
                            texture: asset_server.load("enemy.png"),
                            transform: Transform::from_translation(Vec3::new(
                                path[0].0, path[0].1, 1.0,
                            )),
                            ..Default::default()
                        })
                        .insert(Enemy {
                            health: 4,
                            target_index: 1,
                            path: spawner.last_spawned,
                        });
                    spawner.elapsed = 1.0 / game.level as f32;
                    println!("Time to spawn: {}", spawner.elapsed);
                }
            } else {
                game.level += 1;
                game.timeout = 10.0;
                let enemies_to_spawn = game.level.pow(2) + 10;
                game.enemies_left = enemies_to_spawn;
            }
            spawner.elapsed -= time.delta_seconds();
        }
    } else {
        if enemies.iter().count() == 0 {
            game.timeout -= time.delta_seconds();
        }
        println!("Timeout {}", game.timeout);
    }
}

fn add_spawner(mut commands: Commands) {
    commands.spawn().insert(SpawnTimer {
        last_spawned: 0,
        elapsed: 0.0,
    });
}

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(add_spawner);
        app.add_system(spawn_enemies);
        app.add_system(move_enemies);
    }
}
