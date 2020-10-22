use async_graphql::*;

use crate::get_conn_from_ctx;
use crate::persistence::model::{NewUserEntity, UserEntity};
use crate::persistence::repository;
use crate::utils::{create_token, hash_password, verify};

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

pub struct Query;

#[Object]
impl Query {
    async fn get_users(&self, ctx: &Context<'_>) -> Vec<User> {
        repository::get_all(&get_conn_from_ctx(ctx)).expect("Can't get planets")
            .iter()
            .map(|p| { User::from(p) })
            .collect()
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn create_user(&self, ctx: &Context<'_>, user: UserInput) -> ID {
        let new_user = NewUserEntity {
            username: user.username,
            hash: hash_password(user.password.as_str()).expect("Can't get hash for password"),
            first_name: user.first_name,
            last_name: user.last_name,
            role: user.role,
        };

        let created_user_entity = repository::create(new_user, &get_conn_from_ctx(ctx)).expect("Can't create user");

        created_user_entity.id.into()
    }

    async fn sign_in(&self, ctx: &Context<'_>, sign_in_data: SignInInput) -> String {
        let maybe_user = repository::get_user(&sign_in_data.username, &get_conn_from_ctx(ctx)).ok();

        if let Some(user) = maybe_user {
            if let Ok(matching) = verify(&user.hash, &sign_in_data.password) {
                if matching {
                    return create_token(user);
                }
            }
        }

        panic!("Can't authenticate a user")
    }
}

#[derive(SimpleObject)]
struct User {
    username: String,
    first_name: String,
    last_name: String,
    role: String,
}

#[derive(InputObject)]
struct UserInput {
    username: String,
    password: String,
    first_name: String,
    last_name: String,
    role: String,
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
            role: entity.role.clone(),
        }
    }
}
