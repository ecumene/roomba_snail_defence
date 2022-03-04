use crate::Client;
use crate::Comms;
use crate::Server;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::RwLock;
use warp::ws::{Message, WebSocket};
use warp::Filter;

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

type Users = Arc<RwLock<HashMap<usize, flume::Sender<Message>>>>;

pub async fn run(client: Comms<Client>, server: Comms<Server>) {
    let users = Users::default();
    let server2 = server.clone();
    let users2 = Arc::clone(&users);
    tokio::task::spawn(async move {
        while let Ok(message) = server2.recv.recv_async().await {
            user_message(1, &message, &users2).await;
        }
    });
    let users = warp::any().map(move || {
        let users = users.clone();
        users
    });

    let chat = warp::path("chat")
        .and(warp::ws())
        .and(users)
        .map(move |ws: warp::ws::Ws, users| {
            let client = client.clone();
            ws.on_upgrade(move |socket| user_connected(socket, users, client))
        });

    let assets = warp::path("assets").and(warp::fs::dir("./assets/"));
    let public = warp::path("td").and(warp::fs::dir("./public/"));

    let routes = public.or(assets).or(chat);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn user_connected(ws: WebSocket, users: Users, client: Comms<Client>) {
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    let (tx, rx) = flume::unbounded::<Message>();
    users.write().await.insert(my_id, tx);
    tokio::task::spawn(async move {
        while let Ok(message) = rx.recv_async().await {
            user_ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    });

    while let Some(message) = user_ws_rx.next().await {
        let msg = message.unwrap();
        // todo: cleanup
        if let Ok(text) = msg.to_str() {
            client
                .send
                .send_async(text.to_owned())
                .await
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                });
        }
    }

    user_disconnected(my_id, &users).await;
}

async fn user_message(my_id: usize, message: &str, users: &Users) {
    for (&uid, tx) in users.read().await.iter() {
        if let Err(_disconnected) = tx.send(Message::text(message.clone())) {
            // The tx is disconnected, our `user_disconnected` code
            // should be happening in another task, nothing more to
            // do here.
        }
    }
}

async fn user_disconnected(my_id: usize, users: &Users) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
}
