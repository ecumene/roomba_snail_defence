use crate::plugins::board::Board;
use crate::plugins::board::MAP_SIZE;
use crate::plugins::entities::defense::Turret;
use bevy::prelude::*;
use flume::{Receiver, Sender};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone)]
pub struct Server;
#[derive(Default, Clone)]
pub struct Client;

#[derive(Clone)]
pub struct Comms<T> {
    comm_type: T,
    pub send: Sender<String>,
    pub recv: Receiver<String>,
}

pub struct CommandsPlugin {
    pub server: Comms<Server>,
    pub client: Comms<Client>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BuildType {
    #[serde(rename = "turret")]
    Turret,
    #[serde(rename = "stun")]
    Stun,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildCommand {
    #[serde(rename = "type")]
    pub build_type: BuildType,
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EventType {
    #[serde(rename = "killed")]
    Killed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientEvent {
    #[serde(rename = "type")]
    pub event_type: EventType,
    pub x: u32,
    pub y: u32,
}

pub fn server_comms(input: (Sender<String>, Receiver<String>)) -> Comms<Server> {
    Comms {
        comm_type: Server,
        send: input.0,
        recv: input.1,
    }
}

pub fn client_comms(input: (Sender<String>, Receiver<String>)) -> Comms<Client> {
    Comms {
        comm_type: Client,
        send: input.0,
        recv: input.1,
    }
}

pub fn poll_commands(
    sender: Res<Comms<Server>>,
    reciever: Res<Comms<Client>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if let Ok(message) = reciever.recv.try_recv() {
        commands.spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    right: Val::Px(15.0),
                    ..Rect::default()
                },
                ..Style::default()
            },
            text: Text::with_section(
                message.clone(),
                TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 32.0,
                    color: Color::BLACK,
                },
                TextAlignment::default(),
            ),
            ..TextBundle::default()
        });

        let build_command: BuildCommand = serde_json::from_str(&message).unwrap();
        commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("turret1.png"),
                transform: Transform::from_translation(Vec3::new(
                    build_command.x as f32,
                    build_command.y as f32 + 5.0,
                    10.0,
                )),
                ..Default::default()
            })
            .insert(Turret {
                target: None,
                fire_cooldown: 0.0,
            });

        sender.send.send(message).unwrap();
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
}

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.client.clone());
        app.insert_resource(self.server.clone());
        app.add_startup_system(setup);
        app.add_system(poll_commands);
    }
}
