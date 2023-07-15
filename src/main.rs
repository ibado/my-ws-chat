use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::ws::Message;
use axum::extract::{Query, State};
use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    Json, Router,
};
use futures_util::stream::SplitSink;
use futures_util::{sink::SinkExt, stream::StreamExt};

use crate::messages::MessageRepo;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;

mod messages;

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
pub enum MyMessage {
    InitChat { sender: String, addressee: String },
    Msg { msg: String, author: String },
}

impl MyMessage {
    fn from(str: &str) -> serde_json::Result<Self> {
        serde_json::from_str(str)
    }

    fn as_json_str(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

#[derive(Clone)]
struct MyState {
    client_repo: ClientRepo,
    message_repo: MessageRepo,
}

#[tokio::main]
async fn main() {
    let message_repo = MessageRepo::new()
        .await
        .expect("Error trying to create MessageRepo!");

    let state = MyState {
        client_repo: ClientRepo::new(),
        message_repo,
    };

    let app = Router::new()
        .route("/messages", axum::routing::get(messages_handler))
        .route("/chat", axum::routing::get(chat_handler))
        .with_state(state.clone());

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Deserialize)]
struct Params {
    sender: String,
    addressee: String,
}

async fn messages_handler(
    State(MyState {
        message_repo,
        client_repo: _,
    }): State<MyState>,
    params: Query<Params>,
) -> Json<Vec<MyMessage>> {
    let res = message_repo
        .get_messages(params.0.sender.clone(), params.0.addressee.clone())
        .await
        .into_iter()
        .map(|(m, _)| m)
        .collect();
    Json(res)
}

async fn chat_handler(
    ws: WebSocketUpgrade,
    State(MyState {
        message_repo,
        client_repo,
    }): State<MyState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| on_upgrade(socket, client_repo, message_repo))
}

async fn on_upgrade(socket: WebSocket, clients: ClientRepo, msg_repo: MessageRepo) {
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
                        clients
                            .clients
                            .write()
                            .await
                            .insert(s.clone(), Client { chan: sender_chan });

                        addressee.push_str(&a);
                        sender.push_str(&s);

                        send_stored_msgs(&msg_repo, &mut sender_sock, s, a).await;

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
        while let Some(msg) = receiver_chan.recv().await {
            let _ = sender_sock
                .send(Message::Text(msg.as_json_str().unwrap()))
                .await
                .unwrap();
        }
        eprintln!("Wrong msg, should be MyMessage::Msg");
    });

    let mut receive_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver_sock.next().await {
            if let Message::Text(payload) = msg {
                match MyMessage::from(&payload) {
                    Ok(my_msg) => match my_msg {
                        MyMessage::InitChat { .. } => {
                            eprintln!("wrong msg here. (should be MyMessage::Msg)!");
                        }
                        MyMessage::Msg { msg, .. } => {
                            store_msg(&msg_repo, &sender, &addressee, &msg).await;
                            send_msg_to(&clients, &sender, &sender, &msg).await;
                            send_msg_to(&clients, &addressee, &sender, &msg).await;
                        }
                    },
                    Err(_) => {
                        eprintln!("wrong payload!");
                    }
                }
            }
        }

        println!("Closing socket...");
        clients.clients.write().await.remove(&sender);
    });

    tokio::select! {
        _ = (&mut send_task) => receive_task.abort(),
        _ = (&mut receive_task) => send_task.abort(),
    }
}

async fn send_stored_msgs(
    msg_repo: &MessageRepo,
    sender_sock: &mut SplitSink<WebSocket, Message>,
    sender: String,
    addressee: String,
) {
    let mut msgs = msg_repo
        .get_messages(addressee.clone(), sender.clone())
        .await;
    let mut sender_msgs = msg_repo.get_messages(sender, addressee).await;
    msgs.append(&mut sender_msgs);
    msgs.sort_by(|(_, t1), (_, t2)| t1.partial_cmp(t2).unwrap());
    for (m, _) in msgs.iter() {
        sender_sock
            .send(Message::Text(m.as_json_str().unwrap()))
            .await
            .unwrap();
    }
}

async fn store_msg(msg_repo: &MessageRepo, sender: &str, addressee: &str, msg: &str) {
    msg_repo
        .store_msg(
            MyMessage::Msg {
                msg: msg.to_string(),
                author: sender.to_string(),
            },
            sender.to_string(),
            addressee.to_string(),
        )
        .await;
}

async fn send_msg_to(clients: &ClientRepo, client_key: &str, author: &str, msg: &str) {
    if let Some(Client { chan }) = clients.clients.read().await.get(client_key) {
        chan.send(MyMessage::Msg {
            msg: msg.to_string(),
            author: author.to_string(),
        })
        .unwrap_or_else(|e| eprintln!("channel send error: {e}"));
    } else {
        eprintln!("{client_key} is offline");
    }
}
