#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate strum;

use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse, Result, web};
use actix_web_actors::ws;
use async_graphql::{ID, Schema};
use async_graphql::http::{GQLResponse, GraphQLPlaygroundConfig, playground_source};
use async_graphql_actix_web::{GQLRequest, WSSubscription};
use dataloader::non_cached::Loader;

use dotenv::dotenv;

use crate::graphql::{AppSchema, Details, DetailsBatchLoader, Mutation, Query, Subscription};
use crate::persistence::connection::create_connection_pool;
use crate::persistence::connection::PgPool;

embed_migrations!();

pub mod graphql;
pub mod persistence;

pub async fn index(schema: web::Data<AppSchema>, gql_request: GQLRequest) -> web::Json<GQLResponse> {
    web::Json(GQLResponse(gql_request.into_inner().execute(&schema).await))
}

pub async fn index_ws(schema: web::Data<AppSchema>, req: HttpRequest, payload: web::Payload) -> Result<HttpResponse> {
    ws::start_with_protocols(WSSubscription::new(&schema), &["graphql-ws"], &req, payload)
}

pub async fn index_playground() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"))))
}

pub fn create_schema(pool: PgPool) -> Schema<Query, Mutation, Subscription> {
    let ctx = AppContext::new(pool);
    Schema::build(Query, Mutation, Subscription)
        .limit_complexity(10)
        .limit_depth(5)
        .data(ctx)
        .finish()
}

pub fn prepare_env() -> PgPool {
    dotenv().ok();
    let pool = create_connection_pool();
    let conn = pool.get().expect("Can't get DB connection");
    embedded_migrations::run(&conn);
    pool
}

pub struct AppContext {
    pool: Arc<PgPool>,
    details_batch_loader: Loader<ID, Details, DetailsBatchLoader>,
}

impl AppContext {
    pub fn new(pool: PgPool) -> Self {
        let pool = Arc::new(pool);
        let cloned_pool = Arc::clone(&pool);
        AppContext {
            pool,
            details_batch_loader: Loader::new(DetailsBatchLoader {
                pool: cloned_pool
            }).with_max_batch_size(10),
        }
    }
}
