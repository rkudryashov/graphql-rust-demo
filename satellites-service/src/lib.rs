use actix_web::{web, HttpResponse};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{Context, EmptyMutation, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use diesel_migrations::MigrationHarness;

use crate::graphql::{AppSchema, Query};
use crate::persistence::connection::PgPool;

pub mod graphql;
pub mod persistence;

const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
    diesel_migrations::embed_migrations!("./migrations");

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

pub fn run_migrations(conn: &mut PooledConnection<ConnectionManager<PgConnection>>) {
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");
}

pub fn get_conn_from_ctx(ctx: &Context<'_>) -> PooledConnection<ConnectionManager<PgConnection>> {
    ctx.data::<PgPool>()
        .expect("Can't get pool")
        .get()
        .expect("Can't get DB connection")
}
