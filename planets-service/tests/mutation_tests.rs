use actix_web::{App, guard, test, web};
use jsonpath_lib as jsonpath;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use testcontainers::clients::Cli;

use planets_service::index;

mod common;

#[actix_rt::test]
async fn test_create_planet() {
    let docker = Cli::default();
    let (schema, _pg_container) = common::setup(&docker);

    let mut service = test::init_service(App::new()
        .data(schema.clone())
        .service(web::resource("/").guard(guard::Post()).to(index)))
        .await;

    let mutation = r#"
        mutation ($name: String!, $meanRadius: BigDecimal!, $mass: BigInt!, $population: BigDecimal) {
            createPlanet(
                name: $name
                planetType: TERRESTRIAL_PLANET
                details: {
                    meanRadius: $meanRadius
                    mass: $mass
                    population: $population
                }
            )
        }
        "#.to_string();

    let mut variables = Map::new();
    variables.insert("name".to_string(), "Test planet".into());
    variables.insert("meanRadius".to_string(), "10.7".into());
    variables.insert("mass".to_string(), "6.42e+23".into());
    variables.insert("population".to_string(), "0.5".into());

    let request_body = GraphQLCustomRequest {
        query: mutation,
        variables,
    };

    let request = test::TestRequest::post().uri("/").set_json(&request_body).to_request();

    let response: GraphQLCustomResponse = test::read_response_json(&mut service, request).await;

    let _new_planet_id: i32 = jsonpath::select(&response.data, "$.createPlanet").expect("Can't get satellite by JSON path")[0]
        .as_str().expect("Can't get new planet id")
        .parse().expect("Can't get new planet id");

    // todo get last created planet and check its fields' values
}

#[derive(Serialize)]
struct GraphQLCustomRequest {
    query: String,
    variables: Map<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct GraphQLCustomResponse {
    data: serde_json::Value
}
