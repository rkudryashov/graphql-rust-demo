use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use serde::Deserialize;

use lazy_static::lazy_static;

lazy_static! {
    static ref SECRET_KEY: String = std::env::var("SECRET_KEY").expect("Can't read secret key");
}

pub fn decode_token(token: &str) -> TokenData<Claims> {
    decode::<Claims>(&token, &DecodingKey::from_secret(SECRET_KEY.as_ref()), &Validation::default())
        .expect("Can't decode token")
}

#[derive(Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub role: String,
}
