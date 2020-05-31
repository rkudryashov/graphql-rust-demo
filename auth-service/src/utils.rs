use std::env;

use argonautica::{Error, Hasher, Verifier};

use lazy_static::lazy_static;

lazy_static! {
    static ref SECRET_KEY: String = std::env::var("SECRET_KEY").expect("Can't read secret key");
}

// WARNING: THIS IS ONLY FOR DEMO! PLEASE DO MORE RESEARCH FOR PRODUCTION USE.
pub fn hash_password(password: &str) -> Result<String, Error> {
    Hasher::default()
        .with_password(password)
        .with_secret_key(SECRET_KEY.as_str())
        .hash()
}
