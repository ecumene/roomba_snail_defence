use bevy::prelude::*;
use flume::{Receiver, Sender};

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
    mut commands: Commands,
) {
    if let Ok(message) = reciever.recv.try_recv() {
        println!("from ws:{}", message);
        sender.send.send(message).unwrap();
    }
}

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.client.clone());
        app.insert_resource(self.server.clone());
        app.add_system(poll_commands);
    }
}
