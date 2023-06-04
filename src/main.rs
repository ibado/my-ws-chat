use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::ws::Message;
use axum::extract::State;
use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    Router,
};

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

#[derive(Clone)]
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
    ws.on_upgrade(|socket| process(socket, state))
}

async fn process(mut socket: WebSocket, msgs: ClientRepo) {
    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(_) => return, // client disconnected
        };

        let payload = msg.into_text().unwrap();
        let msg_to_send: MyMessage = MyMessage::from(&payload).unwrap();

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<MyMessage>();
        msgs.clients.write().await.insert(
            msg_to_send.sender.clone(),
            Client {
                chan: tx,
            },
        );

        if let Some(client) = msgs.clients.read().await.get(&msg_to_send.addressee) {
            client.chan.send(msg_to_send.clone()).unwrap();
        };

        let msg_received = rx.recv().await.unwrap();
        socket.send(Message::Text(msg_received.msg)).await.unwrap();
    }
}
