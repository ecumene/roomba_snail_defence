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
            println!("{}", message);
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
            let server = server.clone();
            ws.on_upgrade(move |socket| user_connected(socket, users, client, server))
        });

    let index = warp::path::end().map(|| warp::reply::html(INDEX_HTML));

    let routes = index.or(chat);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn user_connected(ws: WebSocket, users: Users, client: Comms<Client>, server: Comms<Server>) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new chat user: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = flume::unbounded::<Message>();

    // Save the sender in our list of connected users.
    users.write().await.insert(my_id, tx);

    // Spawn a task to send any received messages to the user...
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

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

    // // Every time the user sends a message, broadcast it to
    // // all other users...
    // while let Some(result) = user_ws_rx.next().await {
    //     let msg = match result {
    //         Ok(msg) => {
    //             // Skip any non-Text messages...
    //             if let Ok(s) = msg.to_str() {
    //                 client.send.send(s.to_owned());
    //                 // println!("from server: {}", "test");
    //                 while let Ok(result) = server.recv.recv_async().await {
    //                     // println!("from server: {}", result);
    //                     user_message(my_id, Message::text(result), &users).await;
    //                 }
    //             } else {
    //                 return;
    //             };
    //             msg
    //         }
    //         Err(e) => {
    //             eprintln!("websocket error(uid={}): {}", my_id, e);
    //             break;
    //         }
    //     };
    //     user_message(my_id, msg, &users).await;
    // }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(my_id, &users).await;
}

async fn user_message(my_id: usize, message: &str, users: &Users) {
    let new_msg = format!("<User#{}>: {}", my_id, message);

    for (&uid, tx) in users.read().await.iter() {
        if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
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

static INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <title>Warp Chat</title>
    </head>
    <body>
        <h1>Warp chat</h1>
        <div id="chat">
            <p><em>Connecting...</em></p>
        </div>
        <input type="text" id="text" />
        <button type="button" id="send">Send</button>
        <script type="text/javascript">
        const chat = document.getElementById('chat');
        const text = document.getElementById('text');
        const uri = 'ws://' + location.host + '/chat';
        const ws = new WebSocket(uri);
        function message(data) {
            const line = document.createElement('p');
            line.innerText = data;
            chat.appendChild(line);
        }
        ws.onopen = function() {
            chat.innerHTML = '<p><em>Connected!</em></p>';
        };
        ws.onmessage = function(msg) {
            message(msg.data);
        };
        ws.onclose = function() {
            chat.getElementsByTagName('em')[0].innerText = 'Disconnected!';
        };
        send.onclick = function() {
            const msg = text.value;
            ws.send(msg);
            text.value = '';
            message('<You>: ' + msg);
        };
        </script>
    </body>
</html>
"#;
