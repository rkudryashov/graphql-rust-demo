// WARNING: THIS IS ONLY FOR DEMO! PLEASE DO MORE RESEARCH FOR PRODUCTION USE.
use std::str;

use actix_web::Result;
use argon2::{
    password_hash::{
        rand_core::OsRng, Error, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use chrono::{Duration, Local};
use jsonwebtoken::{encode, EncodingKey, Header};
use lazy_static::lazy_static;

use common_utils::Claims;

use crate::AuthRole;

lazy_static! {
    static ref JWT_SECRET_KEY: String =
        std::env::var("JWT_SECRET_KEY").expect("Can't read JWT_SECRET_KEY");
}

pub fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash_string = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash_string)
}

pub fn verify_password(password_hash_string: &str, input_password: &str) -> Result<(), Error> {
    let parsed_hash = PasswordHash::new(&password_hash_string)?;
    Argon2::default().verify_password(input_password.as_bytes(), &parsed_hash)
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
