#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use actix_web::{web, HttpResponse};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{Context, EmptyMutation, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;

use crate::graphql::{AppSchema, Query};
use crate::persistence::connection::PgPool;

embed_migrations!();

pub mod graphql;
pub mod persistence;

pub fn configure_service(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/")
            .route(web::post().to(index))
            .route(web::get().to(index_playground)),
    );
}

async fn index(schema: web::Data<AppSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn index_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(GraphQLPlaygroundConfig::new("/")))
}

pub fn create_schema_with_context(pool: PgPool) -> Schema<Query, EmptyMutation, EmptySubscription> {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(pool)
        .finish()
}

pub fn run_migrations(pool: &PgPool) {
    let conn = pool.get().expect("Can't get DB connection");
    embedded_migrations::run(&conn).expect("Failed to run database migrations");
}

type Conn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn get_conn_from_ctx(ctx: &Context<'_>) -> Conn {
    ctx.data::<PgPool>()
        .expect("Can't get pool")
        .get()
        .expect("Can't get DB connection")
}
