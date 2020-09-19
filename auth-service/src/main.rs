extern crate auth_service;

use actix_web::{App, guard, HttpServer, web};
use dotenv::dotenv;

use auth_service::{index, index_playground, setup};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let schema = setup();

    HttpServer::new(move || App::new()
        .data(schema.clone())
        .service(web::resource("/").guard(guard::Post()).to(index))
        .service(web::resource("/").guard(guard::Get()).to(index_playground))
    )
        .bind("0.0.0.0:8003")?
        .run()
        .await
}
