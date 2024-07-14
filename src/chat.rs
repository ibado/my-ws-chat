use crate::auth;
use crate::messages::Message;
use crate::messages::MessageRepo;
use crate::users::UserRepo;
use axum::extract::ws::Message as WsMessage;
use axum::extract::ws::WebSocket;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ClientRepo {
    clients: Arc<RwLock<HashMap<u32, Client>>>,
}

impl ClientRepo {
    pub fn new() -> Self {
        Self {
            clients: Arc::default(),
        }
    }
}

#[derive(Clone, Debug)]
struct Client {
    chan: UnboundedSender<Message>,
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

pub async fn on_upgrade(
    socket: WebSocket,
    clients: ClientRepo,
    msg_repo: MessageRepo,
    user_repo: UserRepo,
    jwt_payload: auth::Payload,
) {
    let (mut sender_sock, mut receiver_sock) = socket.split();
    let (sender_chan, receiver_chan) = unbounded_channel::<Message>();

    let mut addressee_id: u32 = 0;
    let sender_id: u32 = jwt_payload.id;
    while let Some(Ok(msg)) = receiver_sock.next().await {
        if let WsMessage::Text(p) = msg {
            match Request::from(&p) {
                Ok(my_message) => {
                    if let Request::InitChat { addressee_nickname } = my_message {
                        if addressee_nickname == jwt_payload.nickname {
                            let failure_msg = Response::ChatInitFailure {
                                error: "You can chat with yourself!".to_string(),
                            }
                            .as_json_str()
                            .unwrap();
                            sender_sock
                                .send(WsMessage::Text(failure_msg))
                                .await
                                .unwrap();
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
                            sender_sock
                                .send(WsMessage::Text(failure_msg))
                                .await
                                .unwrap();
                            break;
                        };
                        clients
                            .clients
                            .write()
                            .await
                            .insert(sender_id, Client { chan: sender_chan });

                        let success_msg = Response::ChatInitSuccess.as_json_str().unwrap();
                        sender_sock
                            .send(WsMessage::Text(success_msg))
                            .await
                            .unwrap();
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

    let mut send_task = tokio::spawn(send_task(receiver_chan, sender_sock, msg_repo.clone()));

    let mut receive_task = tokio::spawn(receive_task(
        receiver_sock,
        msg_repo,
        clients,
        sender_id,
        addressee_id,
    ));

    tokio::select! {
        _ = (&mut send_task) => receive_task.abort(),
        _ = (&mut receive_task) => send_task.abort(),
    }
}

async fn send_task(
    mut receiver_chan: UnboundedReceiver<Message>,
    mut sender_sock: SplitSink<WebSocket, WsMessage>,
    msg_repo: MessageRepo,
) {
    while let Some(Message {
        id,
        payload,
        is_sender,
        sender: _,
    }) = receiver_chan.recv().await
    {
        let response = Response::Msg {
            msg: payload,
            is_sender,
        };
        match response.as_json_str() {
            Ok(json) => {
                if let Err(e) = sender_sock.send(WsMessage::Text(json)).await {
                    eprintln!("Error sending message: {e}");
                } else if !is_sender {
                    msg_repo.message_received(id).await;
                }
            }
            Err(e) => eprintln!("Error parsing message from clinet: {e}"),
        }
    }
}

async fn receive_task(
    mut receiver_sock: futures::stream::SplitStream<WebSocket>,
    msg_repo: MessageRepo,
    clients: ClientRepo,
    sender_id: u32,
    addressee_id: u32,
) {
    while let Some(Ok(msg)) = receiver_sock.next().await {
        if let WsMessage::Text(payload) = msg {
            match Request::from(&payload) {
                Ok(my_msg) => match my_msg {
                    Request::Msg { msg, .. } => {
                        let msg_id = store_msg(&msg_repo, sender_id, addressee_id, &msg).await;
                        tokio::join!(
                            send_msg_to(&clients, sender_id, true, &msg, msg_id),
                            send_msg_to(&clients, addressee_id, false, &msg, msg_id),
                        );
                    }
                    _ => {
                        eprintln!("wrong msg here. (should be Message::Msg)!");
                    }
                },
                Err(e) => {
                    eprintln!("wrong payload: {:?}!", e);
                }
            }
        }
    }

    println!("Closing socket...");
    clients.clients.write().await.remove(&sender_id);
}

async fn send_stored_msgs(
    msg_repo: &MessageRepo,
    sender_sock: &mut SplitSink<WebSocket, WsMessage>,
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
            .send(WsMessage::Text(res.as_json_str().unwrap()))
            .await
            .unwrap();
        msg_repo.message_received(m.id).await;
    }
}

async fn store_msg(msg_repo: &MessageRepo, sender_id: u32, addressee_id: u32, msg: &str) -> u32 {
    msg_repo
        .store_msg(msg.to_string(), sender_id, addressee_id)
        .await
        .unwrap()
}

async fn send_msg_to(
    clients: &ClientRepo,
    client_key: u32,
    is_sender: bool,
    msg: &str,
    msg_id: u32,
) {
    if let Some(Client { chan }) = clients.clients.read().await.get(&client_key) {
        chan.send(Message {
            id: msg_id,
            payload: msg.to_string(),
            is_sender,
            sender: None,
        })
        .unwrap_or_else(|e| eprintln!("channel send error: {e}"));
    } else {
        eprintln!("{client_key} is offline");
    }
}
