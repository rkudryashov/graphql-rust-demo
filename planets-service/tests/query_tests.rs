use actix_web::{test, web, App};
use jsonpath_lib as jsonpath;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use testcontainers::clients::Cli;

use planets_service::{configure_service, create_schema_with_context};

mod common;

const PLANET_FRAGMENT: &str = "
    fragment planetFragment on Planet {
        id
        name
        type
        details {
            meanRadius
            mass
            ... on InhabitedPlanetDetails {
                population
            }
        }
    }
";

#[actix_rt::test]
async fn test_get_planets() {
    let docker = Cli::default();
    let (_pg_container, pool) = common::setup(&docker);

    let service = test::init_service(
        App::new()
            .configure(configure_service)
            .app_data(web::Data::new(create_schema_with_context(pool))),
    )
    .await;

    let query = "
        {
            getPlanets {
                id
                name
                type
                details {
                    meanRadius
                    mass
                    ... on InhabitedPlanetDetails {
                        population
                    }
                }
            }
        }
        "
    .to_string();

    let request_body = GraphQLCustomRequest {
        query,
        variables: Map::new(),
    };

    let request = test::TestRequest::post()
        .uri("/")
        .set_json(&request_body)
        .to_request();

    let response: GraphQLCustomResponse = test::call_and_read_body_json(&service, request).await;

    fn get_planet_as_json(all_planets: &serde_json::Value, index: i32) -> &serde_json::Value {
        jsonpath::select(all_planets, &format!("$.getPlanets[{}]", index))
            .expect("Can't get planet by JSON path")[0]
    }

    let mercury_json = get_planet_as_json(&response.data, 0);
    common::check_planet(mercury_json, 1, "Mercury", "TERRESTRIAL_PLANET", "2439.7");

    let earth_json = get_planet_as_json(&response.data, 2);
    common::check_planet(earth_json, 3, "Earth", "TERRESTRIAL_PLANET", "6371.0");

    let neptune_json = get_planet_as_json(&response.data, 7);
    common::check_planet(neptune_json, 8, "Neptune", "ICE_GIANT", "24622.0");
}

#[actix_rt::test]
async fn test_get_planet_by_id() {
    let docker = Cli::default();
    let (_pg_container, pool) = common::setup(&docker);

    let service = test::init_service(
        App::new()
            .configure(configure_service)
            .app_data(web::Data::new(create_schema_with_context(pool))),
    )
    .await;

    let query = "
        {
            getPlanet(id: 3) {
                ... planetFragment
            }
        }
        "
    .to_string()
        + PLANET_FRAGMENT;

    let request_body = GraphQLCustomRequest {
        query,
        variables: Map::new(),
    };

    let request = test::TestRequest::post()
        .uri("/")
        .set_json(&request_body)
        .to_request();

    let response: GraphQLCustomResponse = test::call_and_read_body_json(&service, request).await;

    let earth_json =
        jsonpath::select(&response.data, "$.getPlanet").expect("Can't get planet by JSON path")[0];
    common::check_planet(earth_json, 3, "Earth", "TERRESTRIAL_PLANET", "6371.0");
}

#[actix_rt::test]
async fn test_get_planet_by_id_with_variable() {
    let docker = Cli::default();
    let (_pg_container, pool) = common::setup(&docker);

    let service = test::init_service(
        App::new()
            .configure(configure_service)
            .app_data(web::Data::new(create_schema_with_context(pool))),
    )
    .await;

    let query = "
        query testPlanetById($planetId: String!) {
            getPlanet(id: $planetId) {
                ... planetFragment
            }
        }"
    .to_string()
        + PLANET_FRAGMENT;

    let jupiter_id = 5;
    let mut variables = Map::new();
    variables.insert("planetId".to_string(), jupiter_id.into());

    let request_body = GraphQLCustomRequest { query, variables };

    let request = test::TestRequest::post()
        .uri("/")
        .set_json(&request_body)
        .to_request();

    let response: GraphQLCustomResponse = test::call_and_read_body_json(&service, request).await;

    let jupiter_json =
        jsonpath::select(&response.data, "$.getPlanet").expect("Can't get planet by JSON path")[0];
    common::check_planet(jupiter_json, 5, "Jupiter", "GAS_GIANT", "69911.0");
}

#[derive(Serialize)]
struct GraphQLCustomRequest {
    query: String,
    variables: Map<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct GraphQLCustomResponse {
    data: serde_json::Value,
}
