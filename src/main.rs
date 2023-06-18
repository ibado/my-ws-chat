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
#[serde(untagged)]
enum MyMessage {
    InitChat { sender: String, addressee: String },
    Msg { msg: String },
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
    let (sender_chan, mut receiver_chan) = tokio::sync::mpsc::unbounded_channel::<MyMessage>();
    let mut addressee = String::new();
    let mut sender = String::new();
    while let Some(Ok(msg)) = receiver_sock.next().await {
        if let Message::Text(p) = msg {
            match MyMessage::from(&p) {
                Ok(my_message) => {
                    if let MyMessage::InitChat {
                        sender: s,
                        addressee: a,
                    } = my_message
                    {
                        msgs.clients
                            .write()
                            .await
                            .insert(s.clone(), Client { chan: sender_chan });

                        addressee.push_str(&a);
                        sender.push_str(&s);

                        sender_sock
                            .send(Message::Text("Nice! You're now connected :D".to_string()))
                            .await
                            .unwrap();
                        break;
                    }
                    eprintln!("wrong msg! (should be InitChat)");
                }
                Err(e) => {
                    eprintln!("wrong payload! Error: {:?}", e);
                }
            }
        }
    }

    let mut send_task = tokio::spawn(async move {
        while let Some(MyMessage::Msg { msg }) = receiver_chan.recv().await {
            let _ = sender_sock.send(Message::Text(msg)).await.unwrap();
        }
        eprintln!("Wrong msg, should be MyMessage::Msg");
    });

    let mut receive_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver_sock.next().await {
            if let Message::Text(payload) = msg {
                match MyMessage::from(&payload) {
                    Ok(MyMessage::Msg { msg }) => {
                        if let Some(Client { chan }) = msgs.clients.read().await.get(&addressee) {
                            chan.send(MyMessage::Msg {
                                msg: format!("{sender}: {msg}"),
                            })
                            .unwrap_or_else(|e| eprintln!("channel send error: {e}"));
                        } else {
                            eprintln!("{addressee} is offline");
                        }
                    }
                    Err(_) => {
                        eprintln!("wrong payload!");
                    }
                    Ok(MyMessage::InitChat { .. }) => {
                        eprintln!("wrong msg here. (should be MyMessage::Msg)!");
                    }
                }
            }
        }

        println!("Closing socket...");
        msgs.clients.write().await.remove(&sender);
    });

    tokio::select! {
        _ = (&mut send_task) => receive_task.abort(),
        _ = (&mut receive_task) => send_task.abort(),
    }
}
