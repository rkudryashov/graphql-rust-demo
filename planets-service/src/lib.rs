#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate strum;

use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse, Result, web};
use actix_web_actors::ws;
use async_graphql::{Context, Schema};
use async_graphql::http::{GQLResponse, GraphQLPlaygroundConfig, playground_source};
use async_graphql_actix_web::{GQLRequest, WSSubscription};
use dataloader::non_cached::Loader;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};

use crate::graphql::{AppSchema, DetailsBatchLoader, Mutation, Query, Subscription};
use crate::persistence::connection::create_connection_pool;
use crate::persistence::connection::PgPool;

embed_migrations!();

pub mod graphql;
mod persistence;

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

pub fn setup() -> Schema<Query, Mutation, Subscription> {
    let pg_pool = prepare_env();
    create_schema(pg_pool)
}

fn create_schema(pool: PgPool) -> Schema<Query, Mutation, Subscription> {
    let pool = Arc::new(pool);
    let cloned_pool = Arc::clone(&pool);
    let details_batch_loader = Loader::new(DetailsBatchLoader {
        pool: cloned_pool
    }).with_max_batch_size(10);

    Schema::build(Query, Mutation, Subscription)
        // limits are commented out, because otherwise introspection query won't work
        // .limit_depth(3)
        // .limit_complexity(15)
        .data(pool)
        .data(details_batch_loader)
        .finish()
}

fn prepare_env() -> PgPool {
    let pool = create_connection_pool();
    let conn = pool.get().expect("Can't get DB connection");
    embedded_migrations::run(&conn);
    pool
}

type Conn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn get_conn_from_ctx(ctx: &Context<'_>) -> Conn {
    ctx.data::<Arc<PgPool>>().expect("Can't get pool").get().expect("Can't get DB connection")
}
