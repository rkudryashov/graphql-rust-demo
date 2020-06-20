#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use actix_web::{HttpResponse, Result, web};
use async_graphql::{EmptySubscription, Schema};
use async_graphql::http::{GQLResponse, GraphQLPlaygroundConfig, playground_source};
use async_graphql_actix_web::GQLRequest;

use dotenv::dotenv;

use crate::graphql::{AppSchema, Mutation, Query};
use crate::persistence::connection::create_connection_pool;
use crate::persistence::connection::PgPool;

embed_migrations!();

mod graphql;
mod persistence;
pub mod utils;

pub async fn index(schema: web::Data<AppSchema>, gql_request: GQLRequest) -> web::Json<GQLResponse> {
    web::Json(GQLResponse(gql_request.into_inner().execute(&schema).await))
}

pub async fn index_playground() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(GraphQLPlaygroundConfig::new("/"))))
}

pub fn create_schema(pool: PgPool) -> Schema<Query, Mutation, EmptySubscription> {
    Schema::build(Query, Mutation, EmptySubscription)
        .enable_federation()
        .data(pool)
        .finish()
}

pub fn prepare_env() -> PgPool {
    dotenv().ok();
    let pool = create_connection_pool();
    let conn = pool.get().expect("Can't get DB connection");
    embedded_migrations::run(&conn);
    pool
}
