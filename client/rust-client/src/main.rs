use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use reqwest::header::AUTHORIZATION;
use reqwest::{header::HeaderValue, header::CONTENT_TYPE, Client};
use std::{collections::HashMap, process::exit};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

const BASE_URL: &'static str = "http://localhost:3000";
const WEB_SOCKET_URL: &'static str = "ws://localhost:3000/chat";

#[tokio::main]
async fn main() {
    let jwt = match menu() {
        Action::Login => {
            let (nick, pass) = take_nick_and_pass();
            login(&nick, &pass).await
        }
        Action::Signup => {
            let (nick, pass) = take_nick_and_pass();
            if sign_up(&nick, &pass).await {
                login(&nick, &pass).await
            } else {
                eprintln!("Error trying to signup!");
                exit(1);
            }
        }
    }
    .unwrap_or_else(|| {
        eprintln!("Error trying to login!");
        exit(1);
    });

    let addressee = take_stdin("Who do you want to chat with?");

    let mut req = WEB_SOCKET_URL.into_client_request().unwrap();
    let auth_value = HeaderValue::from_str(jwt.as_ref()).unwrap();
    req.headers_mut().insert(AUTHORIZATION, auth_value);

    let (socket, _response) = connect_async(req).await.expect("Can't connect");

    let (mut w, mut r) = socket.split();

    w.send(Message::Text(format!(
        "{{\"type\": \"init_chat\", \"addressee_nickname\": \"{addressee}\"}}"
    )))
    .await
    .unwrap();

    let mut reader = tokio::spawn(async move {
        while let Some(Ok(Message::Text(message))) = r.next().await {
            println!("new message: {message}");
        }
    });

    let mut writer = tokio::spawn(async move {
        loop {
            let msg = take_stdin("");
            w.send(Message::Text(format!(
                "{{\"type\": \"msg\", \"msg\": \"{msg}\"}}"
            )))
            .await
            .unwrap();
        }
    });

    tokio::select! {
        _ = (&mut writer) => reader.abort(),
        _ = (&mut reader) => writer.abort(),
    }
}

enum Action {
    Login,
    Signup,
}

fn take_nick_and_pass() -> (String, String) {
    let nick = take_stdin("Nicknake:");
    let pass = take_stdin("Password:");

    (nick, pass)
}

fn menu() -> Action {
    let msg = "Select an option number:\n\t1) Login\n\t2) Signup";
    match take_stdin(msg).as_ref() {
        "1" => Action::Login,
        "2" => Action::Signup,
        i => {
            eprintln!("'{i}' is not a valid option!");
            exit(1);
        }
    }
}

fn take_stdin(msg: &str) -> String {
    println!("{msg}");
    std::io::stdin().lines().take(1).flatten().last().unwrap()
}

async fn login(nickname: &str, password: &str) -> Option<String> {
    Client::new()
        .post(format!("{BASE_URL}/login"))
        .header(CONTENT_TYPE, "application/json")
        .body(format!(
            "{{\"nickname\": \"{nickname}\", \"password\": \"{password}\"}}"
        ))
        .send()
        .await
        .ok()?
        .json::<HashMap<String, String>>()
        .await
        .ok()?
        .get("jwt")
        .map(|jwt| jwt.to_string())
}

async fn sign_up(nickname: &str, password: &str) -> bool {
    Client::new()
        .post(format!("{BASE_URL}/signup"))
        .header(CONTENT_TYPE, "application/json")
        .body(format!(
            "{{\"nickname\": \"{nickname}\", \"password\": \"{password}\"}}"
        ))
        .send()
        .await
        .unwrap()
        .status()
        .as_u16()
        == 201
}
