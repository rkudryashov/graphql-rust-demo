// WARNING: THIS IS ONLY FOR DEMO! PLEASE DO MORE RESEARCH FOR PRODUCTION USE.
use actix_web::Result;
use argonautica::{Error, Hasher, Verifier};
use chrono::{Duration, Local};
use jsonwebtoken::{encode, EncodingKey, Header};
use lazy_static::lazy_static;

use common_utils::Claims;

use crate::AuthRole;

lazy_static! {
    static ref PASSWORD_SECRET_KEY: String =
        std::env::var("PASSWORD_SECRET_KEY").expect("Can't read PASSWORD_SECRET_KEY");
    static ref JWT_SECRET_KEY: String =
        std::env::var("JWT_SECRET_KEY").expect("Can't read JWT_SECRET_KEY");
}

pub fn hash_password(password: &str) -> Result<String, Error> {
    Hasher::default()
        .with_password(password)
        .with_secret_key(PASSWORD_SECRET_KEY.as_str())
        .hash()
}

pub fn verify_password(hash: &str, password: &str) -> Result<bool, Error> {
    Verifier::default()
        .with_hash(hash)
        .with_password(password)
        .with_secret_key(PASSWORD_SECRET_KEY.as_str())
        .verify()
}

pub fn get_jwt_secret_key() -> String {
    JWT_SECRET_KEY.clone()
}

pub fn create_jwt_token(
    username: String,
    role: AuthRole,
    secret_key: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let exp_time = Local::now() + Duration::minutes(60);

    let claims = Claims {
        sub: username,
        exp: exp_time.timestamp(),
        role: role.to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_bytes()),
    )
}
