#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use actix_web::{HttpResponse, Result, web};
use async_graphql::{Context, EmptySubscription, Schema};
use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
use async_graphql_actix_web::{Request, Response};
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};

use crate::graphql::{AppSchema, Mutation, Query};
use crate::persistence::connection::create_connection_pool;
use crate::persistence::connection::PgPool;

embed_migrations!();

pub mod graphql;
pub mod utils;
mod persistence;

pub async fn index(schema: web::Data<AppSchema>, req: Request) -> Response {
    schema.execute(req.into_inner()).await.into()
}

pub async fn index_playground() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(GraphQLPlaygroundConfig::new("/")))
    )
}

pub fn setup() -> Schema<Query, Mutation, EmptySubscription> {
    let pg_pool = prepare_env();
    create_schema(pg_pool)
}

fn create_schema(pool: PgPool) -> Schema<Query, Mutation, EmptySubscription> {
    Schema::build(Query, Mutation, EmptySubscription)
        .enable_federation()
        .data(pool)
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
    ctx.data::<PgPool>().expect("Can't get pool").get().expect("Can't get DB connection")
}
