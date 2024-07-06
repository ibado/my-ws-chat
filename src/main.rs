use auth::extract_jwt;
use axum::{
    extract::{State, WebSocketUpgrade},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json, Router,
};
use notifications::NotificationRepo;
use std::error::Error;
use users::UserRepo;

use crate::chat::ClientRepo;
use crate::{messages::MessageRepo, notifications::notifications_handler};
use serde::{Deserialize, Serialize};
use sqlx::migrate::MigrateDatabase;
use sqlx::{Sqlite, SqlitePool};

mod auth;
mod chat;
mod messages;
mod notifications;
mod types;
mod users;

type AxumResponse = axum::response::Result<axum::response::Response>;

#[derive(Clone)]
pub struct MyState {
    client_repo: chat::ClientRepo,
    message_repo: MessageRepo,
    user_repo: UserRepo,
    notification_repo: NotificationRepo,
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
    if Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        println!("DB is already created, skipping...")
    } else {
        println!("Creating database {}...", &db_url);
        match Sqlite::create_database(&db_url).await {
            Ok(_) => println!("DB created!"),
            Err(error) => panic!("error: {}", error),
        }
    }
    let pool = SqlitePool::connect(&db_url).await?;
    sqlx::migrate!().run(&pool).await?;

    let state = MyState {
        client_repo: ClientRepo::new(),
        message_repo: MessageRepo::new(&pool),
        user_repo: UserRepo::new(&pool),
        notification_repo: NotificationRepo::new(),
    };

    let app = Router::new()
        .route("/chat", axum::routing::get(chat_handler))
        .route("/signup", axum::routing::post(signup_handler))
        .route("/login", axum::routing::post(login_handler))
        .route("/notifications", axum::routing::get(notifications_handler))
        .with_state(state.clone());

    axum::Server::bind(&"0.0.0.0:7777".parse()?)
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
            chat::on_upgrade(
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
