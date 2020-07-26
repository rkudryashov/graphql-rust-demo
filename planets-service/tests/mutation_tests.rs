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

    // get max id of any of the existing planets

    let query = "
        {
            planets {
                id
            }
        }
        ".to_string();

    let request_body = GraphQLCustomRequest {
        query,
        variables: Map::new(),
    };

    let request = test::TestRequest::post().uri("/").set_json(&request_body).to_request();

    let response: GraphQLCustomResponse = test::read_response_json(&mut service, request).await;

    let max_planet_id: i32 = jsonpath::select(&response.data, "$.planets[*].id").expect("Can't get ids")
        .iter()
        .map(|value| value
            .as_str().expect("Can't convert id")
            .parse().expect("Can't convert id")
        )
        .max()
        .expect("Can't get max id");

    // create new planet

    let mutation = r#"
        mutation ($planetName: String!) {
            createPlanet(
                name: $planetName
                planetType: TERRESTRIAL_PLANET
                details: {
                    meanRadius: "10.7"
                    mass: { mantissa: 1.5, exponent: 25 }
                    population: "0.5"
                }
            )
        }
        "#.to_string();

    let planet_name = format!("test_planet_{}", max_planet_id + 1);
    let mut variables = Map::new();
    variables.insert("planetName".to_string(), planet_name.into());

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
