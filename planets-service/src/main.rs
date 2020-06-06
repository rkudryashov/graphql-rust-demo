#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate strum;

use std::sync::Arc;

use actix_web::{App, guard, HttpRequest, HttpResponse, HttpServer, Result, web};
use actix_web_actors::ws;
use async_graphql::{ID, Schema};
use async_graphql::http::{GQLResponse, playground_source};
use async_graphql_actix_web::{GQLRequest, WSSubscription};
use dataloader::non_cached::Loader;

use dotenv::dotenv;
use graphql::{AppSchema, Details, DetailsBatchLoader, Mutation, Query, Subscription};

use crate::persistence::connection::PgPool;

mod graphql;
mod persistence;

embed_migrations!();

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let pool = persistence::connection::create_connection_pool();
    let conn = pool.get().expect("Can't get DB connection");

    embedded_migrations::run(&conn);

    let ctx = AppContext::new(pool);

    let schema = Schema::build(Query, Mutation, Subscription)
        .limit_complexity(10)
        .limit_depth(5)
        .data(ctx)
        .finish();

    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(web::resource("/").guard(guard::Get())
                         .guard(guard::Header("upgrade", "websocket"))
                         .to(index_ws),
            )
            .service(web::resource("/").guard(guard::Get()).to(index_playground))
    })
        .bind("127.0.0.1:8001")?
        .run()
        .await
}

async fn index(schema: web::Data<AppSchema>, gql_request: GQLRequest) -> web::Json<GQLResponse> {
    web::Json(GQLResponse(gql_request.into_inner().execute(&schema).await))
}

async fn index_ws(schema: web::Data<AppSchema>, req: HttpRequest, payload: web::Payload) -> Result<HttpResponse> {
    ws::start_with_protocols(WSSubscription::new(&schema), &["graphql-ws"], &req, payload)
}

async fn index_playground() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source("/", Some("/"))))
}

struct AppContext {
    pool: Arc<PgPool>,
    details_batch_loader: Loader<ID, Details, DetailsBatchLoader>,
}

// todo simplify
impl AppContext {
    fn new(pool: PgPool) -> Self {
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
