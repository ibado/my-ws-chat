use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::ws::Message;
use axum::extract::State;
use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;

#[derive(Clone)]
struct ClientRepo {
    clients: Arc<RwLock<HashMap<String, Client>>>,
}

impl ClientRepo {
    fn new() -> Self {
        Self {
            clients: Arc::default(),
        }
    }
}

#[derive(Clone, Debug)]
struct Client {
    chan: UnboundedSender<MyMessage>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct MyMessage {
    msg: String,
    sender: String,
    addressee: String,
}

impl MyMessage {
    fn from(str: &str) -> serde_json::Result<Self> {
        serde_json::from_str(str)
    }
}

#[tokio::main]
async fn main() {
    let state = ClientRepo::new();
    let app = Router::new()
        .route("/messages", axum::routing::get(messages_handler))
        .with_state(state.clone());

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn messages_handler(
    ws: WebSocketUpgrade,
    State(state): State<ClientRepo>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| on_upgrade(socket, state))
}

async fn on_upgrade(socket: WebSocket, msgs: ClientRepo) {
    let (mut sender_sock, mut receiver_sock) = socket.split();
    // wait first msg to get the sender id/alias/handle/name
    let mut sender = String::new();
    while let Some(Ok(msg)) = receiver_sock.next().await {
        if let Message::Text(p) = msg {
            match MyMessage::from(&p) {
                Ok(s) => {
                    sender.push_str(&s.sender);
                    sender_sock.send(Message::Text("Nice! You're now connected :D".to_string())).await.unwrap();
                    break;
                }
                Err(_) => {
                    eprintln!("wrong payload! (first msg)");
                }
            }
        }
    }
    let (sender_chan, mut receiver_chan) = tokio::sync::mpsc::unbounded_channel::<MyMessage>();
    // persist the client sender channel
    msgs.clients.write().await.insert(
        sender,
        Client {
            chan: sender_chan,
        },
    );

    let mut send_task = tokio::spawn(async move {
        while let Some(m) = receiver_chan.recv().await {
            let _ = sender_sock.send(Message::Text(m.msg)).await.unwrap();
        }
    });

    let mut receive_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver_sock.next().await {
            if let Message::Text(payload) = msg {
                match MyMessage::from(&payload) {
                    Ok(my_msg) => {
                        if let Some(client) = msgs.clients.read().await.get(&my_msg.addressee) {
                            client.chan.send(my_msg.clone()).unwrap();
                        } else {
                            eprintln!("Ups, somthing went wrong!");
                        }
                    },
                    Err(_) => {
                        eprintln!("wrong payload!");
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => receive_task.abort(),
        _ = (&mut receive_task) => send_task.abort(),
    }
}
