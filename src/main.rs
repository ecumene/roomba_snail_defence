use crate::plugins::commands::client_comms;
use crate::plugins::commands::server_comms;
use crate::plugins::commands::Client;
use crate::plugins::commands::Comms;
use crate::plugins::commands::Server;
use crate::plugins::entities::defense::DefensePlugin;
use crate::plugins::entities::effects::FlashPlugin;
use crate::plugins::entities::enemies::EnemiesPlugin;
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use warp::ws::Message;

mod plugins;
mod server;

use plugins::board::BoardPlugin;
use plugins::commands::CommandsPlugin;

#[derive(Component)]
pub struct GameCamera;

pub struct Game {
    pub level: u32,
    pub enemies_left: u32,
    pub timeout: f32,
}

fn camera_system(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection.scale = 0.25;
    commands.spawn_bundle(camera).insert(GameCamera);
}

async fn app(client_comms: Comms<Client>, server_comms: Comms<Server>) {
    App::new()
        .insert_resource(Game {
            level: 1,
            enemies_left: 10,
            timeout: 20.0,
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(camera_system)
        .add_plugin(CommandsPlugin {
            server: server_comms,
            client: client_comms,
        })
        .add_plugin(BoardPlugin)
        .add_plugin(DefensePlugin)
        .add_plugin(FlashPlugin)
        .add_plugin(EnemiesPlugin)
        .run();
}

pub type Users = Arc<RwLock<HashMap<usize, flume::Sender<Message>>>>;

#[tokio::main]
async fn main() {
    let comms_client = client_comms(flume::unbounded::<String>());
    let comms_server = server_comms(flume::unbounded::<String>());

    tokio::spawn(server::run(comms_client.clone(), comms_server.clone()));

    app(comms_client.clone(), comms_server.clone()).await;
}
