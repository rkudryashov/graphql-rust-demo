use std::str::FromStr;

use async_graphql::*;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

use common_utils::{CustomError, FORBIDDEN_MESSAGE};

use crate::persistence::model::{NewUserEntity, UserEntity};
use crate::persistence::repository;
use crate::utils::{create_jwt_token, get_jwt_secret_key, hash_password, verify_password};
use crate::{get_conn_from_ctx, AuthRole};

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

pub struct Query;

#[Object]
impl Query {
    async fn get_users(&self, ctx: &Context<'_>) -> Vec<User> {
        repository::get_all(&get_conn_from_ctx(ctx))
            .expect("Can't get planets")
            .iter()
            .map(User::from)
            .collect()
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    #[graphql(guard = "RoleGuard::new(AuthRole::Admin)")]
    async fn create_user(&self, ctx: &Context<'_>, user: UserInput) -> Result<User> {
        let new_user = NewUserEntity {
            username: user.username,
            hash: hash_password(user.password.as_str())?,
            first_name: user.first_name,
            last_name: user.last_name,
            role: user.role.to_string(),
        };

        let created_user_entity = repository::create(new_user, &get_conn_from_ctx(ctx))?;

        Ok(User::from(&created_user_entity))
    }

    async fn sign_in(&self, ctx: &Context<'_>, input: SignInInput) -> Result<String> {
        let user = repository::get_user(&input.username, &get_conn_from_ctx(ctx))?;

        if verify_password(&user.hash, &input.password)? {
            let role = AuthRole::from_str(user.role.as_str())?;
            let new_token = create_jwt_token(user.username, role, &get_jwt_secret_key())?;
            Ok(new_token)
        } else {
            Err("Can't authenticate a user".into())
        }
    }
}

#[derive(SimpleObject)]
struct User {
    username: String,
    first_name: String,
    last_name: String,
    role: Role,
}

#[derive(InputObject)]
struct UserInput {
    username: String,
    password: String,
    first_name: String,
    last_name: String,
    role: Role,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Enum, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum Role {
    Admin,
    User,
}

#[derive(InputObject)]
struct SignInInput {
    username: String,
    password: String,
}

impl From<&UserEntity> for User {
    fn from(entity: &UserEntity) -> Self {
        User {
            username: entity.username.clone(),
            first_name: entity.first_name.clone(),
            last_name: entity.last_name.clone(),
            role: Role::from_str(entity.role.as_str()).expect("Can't convert &str to Role"),
        }
    }
}

struct RoleGuard {
    role: AuthRole,
}

impl RoleGuard {
    fn new(role: AuthRole) -> Self {
        Self { role }
    }
}

#[async_trait::async_trait]
impl Guard for RoleGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let maybe_getting_role_result = ctx.data_opt::<Result<Option<AuthRole>, CustomError>>();
        match maybe_getting_role_result {
            Some(getting_role_result) => {
                let check_role_result =
                    common_utils::check_user_role_is_allowed(getting_role_result, &self.role);
                match check_role_result {
                    Ok(_) => Ok(()),
                    Err(e) => Err(Error::new(e.message)),
                }
            }
            None => Err(FORBIDDEN_MESSAGE.into()),
        }
    }
}
