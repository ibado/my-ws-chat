use auth::extract_jwt;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json, Router,
};
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, StreamExt},
};
use notifications::NotificationRepo;
use std::{collections::HashMap, error::Error, sync::Arc};
use users::UserRepo;

use crate::{
    messages::MessageRepo,
    notifications::{notifications_handler, Notification},
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedSender},
    RwLock,
};

mod auth;
mod messages;
mod notifications;
mod types;
mod users;

type AxumResponse = axum::response::Result<axum::response::Response>;

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
    chan: UnboundedSender<Response>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    InitChat { addressee_nickname: String },
    Msg { msg: String },
}

impl Request {
    fn from(str: &str) -> serde_json::Result<Self> {
        serde_json::from_str(str)
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    ChatInitSuccess,
    ChatInitFailure { error: String },
    Msg { msg: String, is_sender: bool },
}

impl Response {
    fn as_json_str(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

#[derive(Clone)]
pub struct MyState {
    client_repo: ClientRepo,
    message_repo: MessageRepo,
    user_repo: UserRepo,
    notification_repo: notifications::NotificationRepo,
}

#[derive(Debug, Deserialize, Serialize)]
struct UserReq {
    nickname: String,
    password: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct UserAuthenticated {
    jwt: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var is missing!");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
    sqlx::migrate!().run(&pool).await?;

    let message_repo = MessageRepo::new(&pool);
    let user_repo = UserRepo::new(&pool);

    let state = MyState {
        client_repo: ClientRepo::new(),
        message_repo,
        user_repo,
        notification_repo: notifications::NotificationRepo::new(),
    };

    let app = Router::new()
        .route("/chat", axum::routing::get(chat_handler))
        .route("/signup", axum::routing::post(signup_handler))
        .route("/login", axum::routing::post(login_handler))
        .route("/notifications", axum::routing::get(notifications_handler))
        .with_state(state.clone());

    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn signup_handler(State(state): State<MyState>, body: Json<UserReq>) -> AxumResponse {
    let pass = auth::hash_pass(&body.password)?;
    state.user_repo.store(body.nickname.clone(), pass).await?;
    Ok(StatusCode::CREATED.into_response())
}

async fn login_handler(State(state): State<MyState>, body: Json<UserReq>) -> AxumResponse {
    let user_opt = state.user_repo.get_by_nickname(&body.nickname).await?;
    let res = if let Some(users::User { id, password_hash }) = user_opt {
        if auth::check_pass(&body.password, &password_hash)? {
            let jwt = auth::generate_jwt(id, &body.nickname)?;
            Json(UserAuthenticated { jwt }).into_response()
        } else {
            StatusCode::UNAUTHORIZED.into_response()
        }
    } else {
        StatusCode::NOT_FOUND.into_response()
    };

    Ok(res)
}

async fn chat_handler(
    ws: WebSocketUpgrade,
    State(MyState {
        message_repo,
        client_repo,
        user_repo,
        notification_repo,
    }): State<MyState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    match extract_jwt(headers) {
        Ok(jwt_payload) => ws.on_upgrade(|socket| {
            on_upgrade(
                socket,
                client_repo,
                message_repo,
                user_repo,
                notification_repo,
                jwt_payload,
            )
        }),
        Err(_) => StatusCode::UNAUTHORIZED.into_response(),
    }
}

async fn on_upgrade(
    socket: WebSocket,
    clients: ClientRepo,
    msg_repo: MessageRepo,
    user_repo: UserRepo,
    notification_repo: NotificationRepo,
    jwt_payload: auth::Payload,
) {
    let (mut sender_sock, mut receiver_sock) = socket.split();
    let (sender_chan, mut receiver_chan) = unbounded_channel::<Response>();

    let mut addressee_id: u32 = 0;
    let sender_id: u32 = jwt_payload.id;
    let sender_nickname = jwt_payload.nickname.clone();
    while let Some(Ok(msg)) = receiver_sock.next().await {
        if let Message::Text(p) = msg {
            match Request::from(&p) {
                Ok(my_message) => {
                    if let Request::InitChat { addressee_nickname } = my_message {
                        if addressee_nickname == jwt_payload.nickname {
                            let failure_msg = Response::ChatInitFailure {
                                error: "You can chat with yourself!".to_string(),
                            }
                            .as_json_str()
                            .unwrap();
                            sender_sock.send(Message::Text(failure_msg)).await.unwrap();
                            break;
                        }
                        addressee_id = if let Ok(Some(user)) =
                            user_repo.get_by_nickname(&addressee_nickname).await
                        {
                            user.id
                        } else {
                            let failure_msg = Response::ChatInitFailure {
                                error: "Addressee not found!".to_string(),
                            }
                            .as_json_str()
                            .unwrap();
                            sender_sock.send(Message::Text(failure_msg)).await.unwrap();
                            break;
                        };
                        clients
                            .clients
                            .write()
                            .await
                            .insert(sender_id, Client { chan: sender_chan });

                        let success_msg = Response::ChatInitSuccess.as_json_str().unwrap();
                        sender_sock.send(Message::Text(success_msg)).await.unwrap();

                        send_stored_msgs(&msg_repo, &mut sender_sock, sender_id, addressee_id)
                            .await;

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
        while let Some(ref response) = receiver_chan.recv().await {
            let (msg_payload, is_sender) = match response {
                Response::Msg { msg, is_sender } => (msg, is_sender),
                _ => unreachable!(),
            };
            match response.as_json_str() {
                Ok(json) => {
                    if let Err(e) = sender_sock.send(Message::Text(json)).await {
                        eprintln!("Error sending message: {e}");
                    } else {
                        if *is_sender {
                            if let Some(sender) = notification_repo
                                .notifications
                                .read()
                                .await
                                .get(&addressee_id)
                            {
                                if let Err(e) = sender.chan.send(Notification {
                                    addressee_nickname: sender_nickname.clone(),
                                    message: msg_payload.clone(),
                                }) {
                                    eprintln!("Error trying to send notificaiton: {e}");
                                }
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Error parsing message from clinet: {e}"),
            }
        }
    });

    let mut receive_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver_sock.next().await {
            if let Message::Text(payload) = msg {
                match Request::from(&payload) {
                    Ok(my_msg) => match my_msg {
                        Request::Msg { msg, .. } => {
                            tokio::join!(
                                store_msg(&msg_repo, sender_id, addressee_id, &msg),
                                send_msg_to(&clients, sender_id, true, &msg),
                                send_msg_to(&clients, addressee_id, false, &msg)
                            );
                        }
                        _ => {
                            eprintln!("wrong msg here. (should be MyMessage::Msg)!");
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
    for m in msgs.iter() {
        let res = Response::Msg {
            msg: m.payload.clone(),
            is_sender: m.is_sender,
        };
        sender_sock
            .send(Message::Text(res.as_json_str().unwrap()))
            .await
            .unwrap();
    }
}

async fn store_msg(msg_repo: &MessageRepo, sender_id: u32, addressee_id: u32, msg: &str) {
    msg_repo
        .store_msg(msg.to_string(), sender_id, addressee_id)
        .await;
}

async fn send_msg_to(clients: &ClientRepo, client_key: u32, is_sender: bool, msg: &str) {
    if let Some(Client { chan }) = clients.clients.read().await.get(&client_key) {
        chan.send(Response::Msg {
            msg: msg.to_string(),
            is_sender,
        })
        .unwrap_or_else(|e| eprintln!("channel send error: {e}"));
    } else {
        eprintln!("{client_key} is offline");
    }
}
