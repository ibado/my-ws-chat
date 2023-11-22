use crate::types::Result;
use axum::http::HeaderMap;
use bcrypt::DEFAULT_COST;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub id: u32,
    pub nickname: String,
    pub exp: usize,
}

pub fn hash_pass(pass: &str) -> Result<String> {
    bcrypt::hash(pass, DEFAULT_COST).map_err(|e| eprintln!("Error hashing password: {e}"))
}

pub fn check_pass(pass: &str, hash: &str) -> Result<bool> {
    bcrypt::verify(pass, hash).map_err(|e| eprintln!("Error verifying password: {e}"))
}

pub fn generate_jwt(user_id: u32, nickname: &str) -> Result<String> {
    let now: usize = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| eprintln!("Error getting the system time: {e}"))?
        .as_secs() as usize;
    let exp = now + 60 * 60 * 24 * 90; // 90 days
    let payload = Payload {
        id: user_id,
        nickname: nickname.to_string(),
        exp,
    };

    let jwt = encode(
        &Header::default(),
        &payload,
        &EncodingKey::from_secret("secret".as_ref()),
    )
    .map_err(|e| eprintln!("Error encoding jwt: {e}"))?;

    Ok(jwt.to_string())
}

pub fn decode_jwt(token: String) -> Result<Payload> {
    let dk = DecodingKey::from_secret("secret".as_ref());
    let v = Validation::default();
    decode::<Payload>(&token, &dk, &v)
        .map_err(|e| eprintln!("Error decoding jwt: {e}"))
        .map(|data| data.claims)
}

pub fn extract_jwt(headers: HeaderMap) -> Result<Payload> {
    headers
        .get("Authorization")
        .ok_or_else(|| eprintln!("Missing authorization header."))
        .and_then(|header| {
            header
                .to_str()
                .map(|h| h.to_string().replace("Bearer ", ""))
                .map_err(|e| eprintln!("Error parsing authorization header: {e}"))
        })
        .and_then(|token| crate::auth::decode_jwt(token))
}
