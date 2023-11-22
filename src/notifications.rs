use crate::auth::extract_jwt;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt as _;

use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub addressee_nickname: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Notifier {
    pub chan: UnboundedSender<Notification>,
}

#[derive(Clone)]
pub struct NotificationRepo {
    pub notifications: Arc<RwLock<HashMap<u32, Notifier>>>,
}

impl NotificationRepo {
    pub fn new() -> Self {
        Self {
            notifications: Arc::default(),
        }
    }
}

pub async fn notifications_handler(
    State(state): State<crate::MyState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    match extract_jwt(headers) {
        Ok(jwt_payload) => {
            let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<Notification>();
            state
                .notification_repo
                .notifications
                .write()
                .await
                .insert(jwt_payload.id, Notifier { chan: sender })
                .unwrap();
            let stream = UnboundedReceiverStream::new(receiver)
                .map(|n| serde_json::to_string(&n).map(|s| Event::default().data(s)));
            Sse::new(stream)
                .keep_alive(KeepAlive::default())
                .into_response()
        }
        Err(_) => StatusCode::UNAUTHORIZED.into_response(),
    }
}
