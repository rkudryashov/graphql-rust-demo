use std::env;

use dotenv::dotenv;
use jsonpath_lib as jsonpath;
use testcontainers::clients::Cli;
use testcontainers::images::postgres::Postgres;
use testcontainers::{Container, RunnableImage};

use planets_service::persistence::connection::{create_connection_pool, PgPool};
use planets_service::run_migrations;

pub fn setup(docker: &Cli) -> (Container<Postgres>, PgPool) {
    dotenv().ok();
    let pg_container = setup_database(docker);
    let pool = create_connection_pool();
    run_migrations(&pool);
    (pg_container, pool)
}

fn setup_database(docker: &Cli) -> Container<Postgres> {
    let pg_container = docker.run(get_pg_image());
    let pg_port = pg_container.get_host_port_ipv4(5432);
    env::set_var(
        "DATABASE_URL",
        format!(
            "postgres://postgres:password@localhost:{}/planets-db",
            pg_port
        ),
    );
    pg_container
}

fn get_pg_image() -> RunnableImage<Postgres> {
    RunnableImage::from(Postgres::default())
        .with_env_var(("POSTGRES_DB", "planets-db"))
        .with_env_var(("POSTGRES_PASSWORD", "password"))
}

// TODO: check population
pub fn check_planet(
    planet_json: &serde_json::Value,
    id: i32,
    name: &str,
    type_: &str,
    mean_radius: &str,
) {
    fn check_property(
        planet_json: &serde_json::Value,
        property_name: &str,
        property_expected_value: &str,
    ) {
        let json_path = format!("$.{}", property_name);
        assert_eq!(
            property_expected_value,
            jsonpath::select(&planet_json, &json_path).expect("Can't get property")[0]
                .as_str()
                .expect("Can't get property as str")
        );
    }
    check_property(planet_json, "id", &id.to_string());
    check_property(planet_json, "name", name);
    check_property(planet_json, "type", type_);
    check_property(planet_json, "details.meanRadius", mean_radius);
}
