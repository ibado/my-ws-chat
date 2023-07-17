use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::ws::Message;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    Json, Router,
};
use futures_util::stream::SplitSink;
use futures_util::{sink::SinkExt, stream::StreamExt};
use users::UserRepo;

use crate::messages::MessageRepo;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;
use sqlx::postgres::PgPoolOptions;

mod messages;
mod users;
mod auth;

#[derive(Clone)]
struct ClientRepo {
    clients: Arc<RwLock<HashMap<u32, Client>>>,
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
    InitChat { addressee_nickname: String },
    Msg { msg: String, author_id: u32 },
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
    user_repo: UserRepo,
}

#[tokio::main]
async fn main() {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var is missing!");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();

    let message_repo = MessageRepo::new(&pool);
    let user_repo = UserRepo::new(&pool);

    let state = MyState {
        client_repo: ClientRepo::new(),
        message_repo,
        user_repo,
    };

    let app = Router::new()
        .route("/messages", axum::routing::get(messages_handler))
        .route("/chat", axum::routing::get(chat_handler))
        .route("/signup", axum::routing::post(signup_handler))
        .route("/login", axum::routing::post(login_handler))
        .with_state(state.clone());

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Deserialize, Serialize)]
struct UserReq {
    nickname: String,
    password: String,
}

async fn signup_handler(State(state): State<MyState>, body: Json<UserReq>) -> impl IntoResponse {
    let pass = auth::hash_pass(&body.password);
    if let Some(_) = state.user_repo.store(body.nickname.clone(), pass).await {
        StatusCode::CREATED.into_response()
    } else {
        StatusCode::BAD_REQUEST.into_response()
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct UserAuthenticated {
    jwt: String,
}

async fn login_handler(State(state): State<MyState>, body: Json<UserReq>) -> impl IntoResponse {
    if let Some(users::User { id, password_hash }) = state.user_repo.get_by_nickname(&body.nickname).await {
        if auth::check_pass(&body.password, &password_hash) {
            let jwt = auth::generate_jwt(id, &body.nickname);
            Json(UserAuthenticated { jwt }).into_response()
        } else {
            StatusCode::BAD_REQUEST.into_response()
        }
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

#[derive(Debug, Deserialize)]
struct Params {
    sender_id: u32,
    addressee_id: u32,
}

async fn messages_handler(
    State(state): State<MyState>,
    params: Query<Params>,
) -> Json<Vec<MyMessage>> {
    let res = state.message_repo
        .get_messages(params.0.sender_id, params.0.addressee_id)
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
        user_repo,
    }): State<MyState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Some(h) = headers.get("Authorization") {
        if let Ok(jwt) = h.to_str().map(|h| h.to_string().replace("Bearer ", "")) {
            println!("jwt: {jwt}");
            let jwt_payload = auth::decode_jwt(jwt).unwrap();
            ws.on_upgrade(|socket| on_upgrade(socket, client_repo, message_repo, user_repo, jwt_payload))
        } else {
            StatusCode::UNAUTHORIZED.into_response()
        }
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

async fn on_upgrade(
    socket: WebSocket,
    clients: ClientRepo,
    msg_repo: MessageRepo,
    user_repo: UserRepo,
    jwt_payload: auth::Payload,
) {
    let (mut sender_sock, mut receiver_sock) = socket.split();
    let (sender_chan, mut receiver_chan) = tokio::sync::mpsc::unbounded_channel::<MyMessage>();

    let mut addressee_id: u32 = 0;
    let sender_id: u32 = jwt_payload.id;
    while let Some(Ok(msg)) = receiver_sock.next().await {
        if let Message::Text(p) = msg {
            match MyMessage::from(&p) {
                Ok(my_message) => {
                    if let MyMessage::InitChat {
                        addressee_nickname,
                    } = my_message
                    {
                        addressee_id = if let Some(user) = user_repo.get_by_nickname(&addressee_nickname).await {
                            user.id
                        } else {
                            break;
                        };
                        clients
                            .clients
                            .write()
                            .await
                            .insert(sender_id, Client { chan: sender_chan });

                        send_stored_msgs(&msg_repo, &mut sender_sock, sender_id, addressee_id).await;

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
                            store_msg(&msg_repo, sender_id, addressee_id, &msg).await;
                            send_msg_to(&clients, sender_id, addressee_id, &msg).await;
                            send_msg_to(&clients, addressee_id, sender_id, &msg).await;
                        }
                    },
                    Err(_) => {
                        eprintln!("wrong payload!");
                    }
                }
            }
        }

        println!("Closing socket...");
        clients.clients.write().await.remove(&sender_id);
    });

    tokio::select! {
        _ = (&mut send_task) => receive_task.abort(),
        _ = (&mut receive_task) => send_task.abort(),
    }
}

async fn send_stored_msgs(
    msg_repo: &MessageRepo,
    sender_sock: &mut SplitSink<WebSocket, Message>,
    sender_id: u32,
    addressee_id: u32,
) {
    let msgs = msg_repo.get_messages(sender_id, addressee_id).await;
    for (m, _) in msgs.iter() {
        sender_sock
            .send(Message::Text(m.as_json_str().unwrap()))
            .await
            .unwrap();
    }
}

async fn store_msg(msg_repo: &MessageRepo, sender_id: u32, addressee_id: u32, msg: &str) {
    msg_repo
        .store_msg(
            MyMessage::Msg {
                msg: msg.to_string(),
                author_id: sender_id,
            },
            sender_id,
            addressee_id,
        )
        .await;
}

async fn send_msg_to(clients: &ClientRepo, client_key: u32, author_id: u32, msg: &str) {
    if let Some(Client { chan }) = clients.clients.read().await.get(&client_key) {
        chan.send(MyMessage::Msg {
            msg: msg.to_string(),
            author_id,
        })
        .unwrap_or_else(|e| eprintln!("channel send error: {e}"));
    } else {
        eprintln!("{client_key} is offline");
    }
}
