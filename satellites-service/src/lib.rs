#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate strum;

use std::str::FromStr;

use actix_web::{HttpRequest, HttpResponse, Result, web};
use async_graphql::{Context, EmptyMutation, EmptySubscription, Schema};
use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
use async_graphql_actix_web::{Request, Response};
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use strum_macros::EnumString;

use crate::graphql::{AppSchema, Query};
use crate::persistence::connection::create_connection_pool;
use crate::persistence::connection::PgPool;

embed_migrations!();

pub mod graphql;
mod persistence;
mod utils;

pub async fn index(schema: web::Data<AppSchema>, http_req: HttpRequest, req: Request) -> Response {
    let mut query = req.into_inner();

    let maybe_role = get_role(http_req);
    if let Some(role) = maybe_role {
        query = query.data(role);
    }

    schema.execute(query).await.into()
}

pub async fn index_playground() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(GraphQLPlaygroundConfig::new("/")))
    )
}

pub fn setup() -> Schema<Query, EmptyMutation, EmptySubscription> {
    let pg_pool = prepare_env();
    create_schema(pg_pool)
}

fn create_schema(pool: PgPool) -> Schema<Query, EmptyMutation, EmptySubscription> {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(pool)
        .finish()
}

fn prepare_env() -> PgPool {
    let pool = create_connection_pool();
    let conn = pool.get().expect("Can't get DB connection");
    embedded_migrations::run(&conn);
    pool
}

fn get_role(http_request: HttpRequest) -> Option<Role> {
    http_request
        .headers()
        .get("Authorization")
        .and_then(|header_value| header_value.to_str().ok().map(|s| {
            let jwt_start_index = "Bearer ".len();
            let jwt = s[jwt_start_index..s.len()].to_string();
            let token_data = utils::decode_token(&jwt);
            Role::from_str(&token_data.claims.role).expect("Can't parse role")
        }))
}

#[derive(EnumString)]
#[derive(Eq, PartialEq)]
enum Role {
    Admin,
    User,
}

type Conn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn get_conn_from_ctx(ctx: &Context<'_>) -> Conn {
    ctx.data::<PgPool>().expect("Can't get pool").get().expect("Can't get DB connection")
}
