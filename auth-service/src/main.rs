extern crate auth_service;

use std::env;

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;

use auth_service::persistence::connection::create_connection_pool;
use auth_service::{configure_service, create_schema_with_context, run_migrations};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let pool = create_connection_pool();
    run_migrations(&mut pool.get().expect("Can't get DB connection"));

    let schema = web::Data::new(create_schema_with_context(pool));

    let server_port = env::var("SERVER_PORT").expect("Can't get server port");

    HttpServer::new(move || {
        App::new()
            .configure(configure_service)
            .app_data(schema.clone())
    })
    .bind(format!("0.0.0.0:{}", server_port))?
    .run()
    .await
}
