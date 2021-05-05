use std::env;

use actix_web::{test, App};
use jsonpath_lib as jsonpath;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use testcontainers::clients::Cli;

use planets_service::{configure_service, create_schema_with_context};

mod common;

#[actix_rt::test]
async fn test_create_planet() {
    env::set_var("DISABLE_AUTH", true.to_string());
    let docker = Cli::default();
    let (_pg_container, pool) = common::setup(&docker);

    let mut service = test::init_service(
        App::new()
            .configure(configure_service)
            .data(create_schema_with_context(pool)),
    )
    .await;

    let mutation = r#"
        mutation(
            $name: String!
            $type: PlanetType!
            $meanRadius: BigDecimal!
            $mass: BigInt!
            $population: BigDecimal
        ) {
            createPlanet(
                planet: {
                    name: $name
                    type: $type
                    details: { meanRadius: $meanRadius, mass: $mass, population: $population }
                }
            ) {
                id
                name
                type
                details {
                    meanRadius
                    mass
                }
            }
        }
        "#
    .to_string();

    let mut variables = Map::new();
    variables.insert("name".to_string(), "Test planet".into());
    variables.insert("type".to_string(), "ICE_GIANT".into());
    variables.insert("meanRadius".to_string(), "10.7".into());
    variables.insert("mass".to_string(), "6.42e+23".into());
    variables.insert("population".to_string(), "0.5".into());

    let request_body = GraphQLCustomRequest {
        query: mutation,
        variables,
    };

    let request = test::TestRequest::post()
        .uri("/")
        .set_json(&request_body)
        .to_request();

    let response: GraphQLCustomResponse = test::read_response_json(&mut service, request).await;

    let response_data = response.data.expect("Response doesn't contain data");

    let created_planet_json = jsonpath::select(&response_data, "$.createPlanet")
        .expect("Can't get created planet by JSON path")[0];

    common::check_planet(created_planet_json, 9, "Test planet", "ICE_GIANT", "10.7");
}

#[derive(Serialize)]
struct GraphQLCustomRequest {
    query: String,
    variables: Map<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct GraphQLCustomResponse {
    data: Option<serde_json::Value>,
    #[allow(dead_code)]
    errors: Option<serde_json::Value>,
}
