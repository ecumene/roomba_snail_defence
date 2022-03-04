use crate::plugins::entities::enemies::Enemy;
use crate::GameCamera;
use bevy::ecs::event::Events;
use bevy::prelude::*;
use bevy::window::WindowResized;

pub struct BoardPlugin;

#[derive(Component)]
pub struct Board;

pub const MAP_SIZE: (u32, u32) = (1024, 254);
pub const CAMERA_SCALE: f32 = 0.25;

fn hello_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture = asset_server.load("map-layered.png");
    commands
        .spawn_bundle(SpriteBundle {
            texture: texture,
            transform: Transform::from_translation(Vec3::new(
                MAP_SIZE.0 as f32 / 2.0,
                MAP_SIZE.1 as f32 / 2.0,
                10.0,
            )),
            ..Default::default()
        })
        .insert(Board);
}

fn resize_and_pan(
    time: Res<Time>,
    mut windows: ResMut<Windows>,
    mut query: Query<(&GameCamera, &mut GlobalTransform, Without<Enemy>)>,
    enemies: Query<(&Enemy, &GlobalTransform, Without<GameCamera>)>,
) {
    let window = windows.get_primary_mut().unwrap();
    let mut target_cam_y = 0.0;
    let mut target_cam_x = 0.0;
    for (_, transform, _) in enemies.iter() {
        if target_cam_x < transform.translation.x {
            target_cam_x = transform.translation.x;
        }
        target_cam_y = (target_cam_y + transform.translation.y) / 2.0;
    }

    for (_, mut transform, _) in query.iter_mut() {
        let width_offset = transform.translation.x
            + (target_cam_x - transform.translation.x) * time.delta_seconds();
        let height_offset = transform.translation.y
            + (target_cam_y - transform.translation.y) * time.delta_seconds();
        transform.translation = Vec3::new(width_offset, height_offset, transform.translation.z);
    }
}

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(hello_world);
        app.add_system(resize_and_pan);
        app.insert_resource(ClearColor(Color::rgb(
            0.862745098,
            0.839215686,
            0.729411765,
        )));
    }
}
