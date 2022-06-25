// WARNING: THIS IS ONLY FOR DEMO! PLEASE DO MORE RESEARCH FOR PRODUCTION USE.
use std::str::FromStr;

use actix_web::http::header::ToStrError;
use actix_web::HttpRequest;
use serde::{Deserialize, Serialize};
use strum::ParseError;
use strum_macros::{Display, EnumString};

pub const FORBIDDEN_MESSAGE: &str = "Forbidden";

#[derive(Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub role: String,
}

#[derive(Eq, PartialEq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum Role {
    Admin,
    User,
}

pub fn get_role(http_request: HttpRequest) -> Result<Option<Role>, CustomError> {
    let role_header_value = http_request.headers().get("role");

    match role_header_value {
        Some(header_value) => {
            let header_str = header_value.to_str()?;
            Ok(Some(Role::from_str(header_str)?))
        }
        None => Ok(None),
    }
}

pub fn check_user_role_is_allowed(
    getting_role_result: &Result<Option<Role>, CustomError>,
    allowed_role: &Role,
) -> Result<(), CustomError> {
    let maybe_role = match getting_role_result {
        Ok(maybe_role) => maybe_role,
        Err(e) => {
            return Err(format!("Error while getting a user's role: {}", e.message)
                .as_str()
                .into())
        }
    };

    match maybe_role {
        Some(role) => {
            if role == allowed_role {
                Ok(())
            } else {
                Err(FORBIDDEN_MESSAGE.into())
            }
        }
        None => Err(FORBIDDEN_MESSAGE.into()),
    }
}

#[derive(Debug)]
pub struct CustomError {
    pub message: String,
}

impl From<ToStrError> for CustomError {
    fn from(source: ToStrError) -> Self {
        Self {
            message: source.to_string(),
        }
    }
}

impl From<ParseError> for CustomError {
    fn from(source: ParseError) -> Self {
        Self {
            message: source.to_string(),
        }
    }
}

impl From<&str> for CustomError {
    fn from(source: &str) -> Self {
        Self {
            message: String::from(source),
        }
    }
}
