#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate strum;

use actix_web::{App, guard, HttpRequest, HttpResponse, HttpServer, Result, web};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql::http::{GQLResponse, GraphQLPlaygroundConfig, playground_source};
use async_graphql_actix_web::GQLRequest;

use dotenv::dotenv;
use graphql::{AppSchema, Query};

mod graphql;
mod persistence;
mod utils;

embed_migrations!();

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let pool = persistence::connection::create_connection_pool();
    let conn = pool.get().expect("Can't get DB connection");

    embedded_migrations::run(&conn);

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(pool)
        .finish();

    HttpServer::new(move || App::new()
        .data(schema.clone())
        .service(web::resource("/").guard(guard::Post()).to(index))
        .service(web::resource("/").guard(guard::Get()).to(index_playground))
    )
        .bind("127.0.0.1:8002")?
        .run()
        .await
}

async fn index(schema: web::Data<AppSchema>, http_request: HttpRequest, gql_request: GQLRequest) -> web::Json<GQLResponse> {
    let token = http_request
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok().map(|s| s.to_string()));

    let request_context = RequestContext { token };

    let query = gql_request.into_inner().data(request_context);

    web::Json(GQLResponse(query.execute(&schema).await))
}

async fn index_playground() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(GraphQLPlaygroundConfig::new("/"))))
}

struct RequestContext {
    token: Option<String>
}
