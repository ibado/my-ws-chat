use axum::extract::ws::Message;
use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    Router,
};

use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/messages", axum::routing::get(messages_handler));

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn messages_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| process(socket))
}

#[derive(Deserialize, Serialize)]
struct MyMessage {
    msg: String,
    name: String,
}

impl MyMessage {
    fn from(str: &str) -> serde_json::Result<Self> {
        serde_json::from_str(str)
    }
}

async fn process(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(_) => return, // client disconnected
        };

        let payload = msg.into_text().unwrap();
        let mm: MyMessage = MyMessage::from(&payload).unwrap();
        let res = "hi ".to_string() + &mm.name;

        if socket.send(Message::Text(res)).await.is_err() {
            return; // client disconnected
        }
    }
}
