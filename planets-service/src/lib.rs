#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate strum;

use std::sync::{Arc, Mutex};

use actix_web::{guard, web, HttpRequest, HttpResponse, Result};
use async_graphql::dataloader::DataLoader;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{Context, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;

use crate::graphql::{AppSchema, DetailsLoader, Mutation, Query, Subscription};
use crate::persistence::connection::PgPool;

embed_migrations!();

pub mod graphql;
pub mod kafka;
pub mod persistence;

pub fn configure_service(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/")
            .route(web::post().to(index))
            .route(
                web::get()
                    .guard(guard::Header("upgrade", "websocket"))
                    .to(index_ws),
            )
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

async fn index_ws(
    schema: web::Data<AppSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
}

async fn index_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
        ))
}

pub fn create_schema_with_context(pool: PgPool) -> Schema<Query, Mutation, Subscription> {
    let arc_pool = Arc::new(pool);
    let cloned_pool = Arc::clone(&arc_pool);
    let details_data_loader =
        DataLoader::new(DetailsLoader { pool: cloned_pool }, actix_rt::spawn).max_batch_size(10);

    let kafka_consumer_counter = Mutex::new(0);

    Schema::build(Query, Mutation, Subscription)
        // limits are commented out, because otherwise introspection query won't work
        // .limit_depth(3)
        // .limit_complexity(15)
        .data(arc_pool)
        .data(details_data_loader)
        .data(kafka::create_producer())
        .data(kafka_consumer_counter)
        .enable_subscription_in_federation()
        .finish()
}

pub fn run_migrations(pool: &PgPool) {
    let conn = pool.get().expect("Can't get DB connection");
    embedded_migrations::run(&conn).expect("Failed to run database migrations");
}

type Conn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn get_conn_from_ctx(ctx: &Context<'_>) -> Conn {
    ctx.data::<Arc<PgPool>>()
        .expect("Can't get pool")
        .get()
        .expect("Can't get DB connection")
}
