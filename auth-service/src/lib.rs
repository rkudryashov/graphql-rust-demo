#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use std::str::FromStr;

use actix_web::{HttpRequest, HttpResponse, web};
use async_graphql::{Context, EmptySubscription, Schema};
use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
use async_graphql_actix_web::{Request, Response};
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};

use crate::graphql::{AppSchema, Mutation, Query, Role};
use crate::persistence::connection::PgPool;
use crate::persistence::repository;

embed_migrations!();

pub mod graphql;
pub mod utils;
pub mod persistence;

pub fn configure_service(cfg: &mut web::ServiceConfig) {
    cfg
        .service(web::resource("/")
            .route(web::post().to(index))
            .route(web::get().to(index_playground))
        );
}

async fn index(schema: web::Data<AppSchema>, http_req: HttpRequest, req: Request) -> Response {
    let mut query = req.into_inner();

    let maybe_role = get_role(http_req);
    if let Some(role) = maybe_role {
        query = query.data(role);
    }

    schema.execute(query).await.into()
}

async fn index_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(GraphQLPlaygroundConfig::new("/")))
}

pub fn create_schema_with_context(pool: PgPool) -> Schema<Query, Mutation, EmptySubscription> {
    Schema::build(Query, Mutation, EmptySubscription)
        .enable_federation()
        .data(pool)
        .finish()
}

pub fn run_migrations(pool: &PgPool) {
    let conn = pool.get().expect("Can't get DB connection");
    embedded_migrations::run(&conn).expect("Failed to run database migrations");
    // if environment variable is set (in case of production environment), then update users' hash
    if let Ok(hash) = std::env::var("SECURED_USER_PASSWORD_HASH") {
        repository::update_password_hash(hash, &conn);
    };
}

type Conn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn get_conn_from_ctx(ctx: &Context<'_>) -> Conn {
    ctx.data::<PgPool>().expect("Can't get pool").get().expect("Can't get DB connection")
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
