use actix_web::{web, HttpRequest, HttpResponse};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{Context, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use diesel_migrations::MigrationHarness;

use crate::graphql::{AppSchema, Mutation, Query};
use crate::persistence::connection::PgPool;
use crate::persistence::repository;

pub mod graphql;
pub mod persistence;
mod utils;

const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
    diesel_migrations::embed_migrations!("./migrations");

type AuthRole = common_utils::Role;

pub fn configure_service(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/")
            .route(web::post().to(index))
            .route(web::get().to(index_playground)),
    );
}

async fn index(
    schema: web::Data<AppSchema>,
    http_req: HttpRequest,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut query = req.into_inner();
    let getting_role_result = common_utils::get_role(http_req);
    query = query.data(getting_role_result);
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

pub fn run_migrations(conn: &mut PooledConnection<ConnectionManager<PgConnection>>) {
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");

    // if environment variable is set (in case of production environment), then update users' hash
    if let Ok(hash) = std::env::var("SECURED_USER_PASSWORD_HASH") {
        repository::update_password_hash(hash, conn).expect("Failed to update password hash");
    };
}

pub fn get_conn_from_ctx(ctx: &Context<'_>) -> PooledConnection<ConnectionManager<PgConnection>> {
    ctx.data::<PgPool>()
        .expect("Can't get pool")
        .get()
        .expect("Can't get DB connection")
}
