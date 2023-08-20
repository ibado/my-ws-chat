use reqwest::{blocking::Client, header::CONTENT_TYPE};
use std::{collections::HashMap, process::exit};

const BASE_URL: &'static str = "http://localhost:3000";

fn main() {
    let jwt = match menu() {
        Action::Login => {
            let (nick, pass) = take_nick_and_pass();
            login(&nick, &pass)
        }
        Action::Signup => {
            let (nick, pass) = take_nick_and_pass();
            if sign_up(&nick, &pass) {
                login(&nick, &pass)
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

    println!("JWT is {jwt}");
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

fn login(nickname: &str, password: &str) -> Option<String> {
    Client::new()
        .post(format!("{BASE_URL}/login"))
        .header(CONTENT_TYPE, "application/json")
        .body(format!(
            "{{\"nickname\": \"{nickname}\", \"password\": \"{password}\"}}"
        ))
        .send()
        .and_then(|r| r.json::<HashMap<String, String>>())
        .ok()
        .and_then(|j| j.get("jwt").map(|jwt| jwt.to_string()))
}

fn sign_up(nickname: &str, password: &str) -> bool {
    Client::new()
        .post(format!("{BASE_URL}/signup"))
        .header(CONTENT_TYPE, "application/json")
        .body(format!(
            "{{\"nickname\": \"{nickname}\", \"password\": \"{password}\"}}"
        ))
        .send()
        .map(|r| r.status().as_u16() == 201)
        .unwrap_or(false)
}
