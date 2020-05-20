#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use actix_web::{App, guard, HttpResponse, HttpServer, Result, web};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql::http::{GQLResponse, playground_source};
use async_graphql_actix_web::GQLRequest;

use dotenv::dotenv;
use graphql::{Query, TestSchema};

mod db;
mod db_connection;
mod repository;
mod schema;
mod graphql;
mod model;

embed_migrations!();

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let pool = db_connection::create_connection_pool();
    let conn = pool.get().expect("Can't get DB connection");

    embedded_migrations::run(&conn);

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(pool)
        .finish();

    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(web::resource("/").guard(guard::Get()).to(index_playground))
    })
        .bind("127.0.0.1:8001")?
        .run()
        .await
}

async fn index(
    schema: web::Data<TestSchema>,
    gql_request: GQLRequest,
) -> web::Json<GQLResponse> {
    web::Json(GQLResponse(gql_request.into_inner().execute(&schema).await))
}

async fn index_playground() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source("/", Some("/"))))
}
