#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate strum;

use actix_web::{HttpRequest, HttpResponse, Result, web};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql::http::{GQLResponse, GraphQLPlaygroundConfig, playground_source};
use async_graphql_actix_web::GQLRequest;

use dotenv::dotenv;

use crate::graphql::{AppSchema, Query};
use crate::persistence::connection::create_connection_pool;
use crate::persistence::connection::PgPool;

embed_migrations!();

mod graphql;
mod persistence;
mod utils;

pub async fn index(schema: web::Data<AppSchema>, http_request: HttpRequest, gql_request: GQLRequest) -> web::Json<GQLResponse> {
    let token = http_request
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok().map(|s| s.to_string()));

    let request_context = RequestContext { token };

    let query = gql_request.into_inner().data(request_context);

    web::Json(GQLResponse(query.execute(&schema).await))
}

pub async fn index_playground() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(GraphQLPlaygroundConfig::new("/"))))
}

pub fn create_schema(pool: PgPool) -> Schema<Query, EmptyMutation, EmptySubscription> {
    Schema::build(Query, EmptyMutation, EmptySubscription)
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

struct RequestContext {
    token: Option<String>
}
