// WARNING: THIS IS ONLY FOR DEMO! PLEASE DO MORE RESEARCH FOR PRODUCTION USE.

use argonautica::{Error, Hasher, Verifier};
use chrono::{Duration, Local};
use jsonwebtoken::{decode, DecodingKey, encode, EncodingKey, Header, TokenData, Validation};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::persistence::model::UserEntity;

lazy_static! {
    static ref SECRET_KEY: String = std::env::var("SECRET_KEY").expect("Can't read secret key");
}

// Argon2 stuff
pub fn hash_password(password: &str) -> Result<String, Error> {
    Hasher::default()
        .with_password(password)
        .with_secret_key(SECRET_KEY.as_str())
        .hash()
}

pub fn verify(hash: &str, password: &str) -> Result<bool, Error> {
    Verifier::default()
        .with_hash(hash)
        .with_password(password)
        .with_secret_key(SECRET_KEY.as_str())
        .verify()
}

// JWT stuff
pub fn create_token(user: UserEntity) -> String {
    let exp_time = Local::now() + Duration::minutes(60);

    let claims = Claims {
        sub: user.username,
        exp: exp_time.timestamp(),
        role: user.role,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET_KEY.as_ref()))
        .expect("Can't create token")
}

pub fn decode_token(token: &str) -> TokenData<Claims> {
    decode::<Claims>(&token, &DecodingKey::from_secret(SECRET_KEY.as_ref()), &Validation::default())
        .expect("Can't decode token")
}

#[derive(Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub role: String,
}
