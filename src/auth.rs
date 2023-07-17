use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use bcrypt::{DEFAULT_COST, hash, verify};

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub id: u32,
    pub nickname: String,
    pub exp: usize,
}

pub fn hash_pass(pass: &str) -> String {
    hash(pass, DEFAULT_COST).unwrap()
}

pub fn check_pass(pass: &str, hash: &str) -> bool {
    verify(pass, hash).unwrap()
}

pub fn generate_jwt(user_id: u32, nickname: &str) -> String {
    let now: usize = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let exp = now + 60 * 60 * 24 * 90; // 90 days
    let payload = Payload { id: user_id, nickname: nickname.to_string(), exp };
    let jwt = encode(&Header::default(), &payload, &EncodingKey::from_secret("secret".as_ref())).unwrap();
    jwt.to_string()
}

pub fn decode_jwt(token: String) -> Option<Payload> {
    let dk = DecodingKey::from_secret("secret".as_ref());
    let v = Validation::default();
    decode::<Payload>(&token, &dk, &v)
        .ok()
        .map(|data| data.claims)
}
