extern crate planets_service;

use actix_web::{App, HttpServer};
use dotenv::dotenv;

use planets_service::{configure_service, create_schema_with_context, run_migrations};
use planets_service::persistence::connection::create_connection_pool;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let pool = create_connection_pool();
    run_migrations(&pool);

    HttpServer::new(move || App::new()
        .configure(configure_service)
        .data(create_schema_with_context(pool.clone()))
    )
        .bind("0.0.0.0:8001")?
        .run()
        .await
}
