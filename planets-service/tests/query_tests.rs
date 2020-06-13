use actix_web::{App, guard, test, web};
use jsonpath_lib as jsonpath;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use planets_service::{create_schema, index, prepare_env};

#[actix_rt::test]
async fn test_planets() {
    let pool = prepare_env();
    let schema = create_schema(pool);

    let mut service = test::init_service(App::new()
        .data(schema.clone())
        .service(web::resource("/").guard(guard::Post()).to(index)))
        .await;

    let query = "
            {
                planets {
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
        ".to_string();

    let request_body = GraphQLCustomRequest {
        query,
        // variables: Option::None,
        operation_name: "query".to_string(),
    };

    let request = test::TestRequest::post().uri("/").set_json(&request_body).to_request();

    let result: GraphQLCustomResponse = test::read_response_json(&mut service, request).await;

    fn get_planet_as_json(all_planets: &Value, index: i32) -> &Value {
        jsonpath::select(all_planets, &format!("$..planets[{}]", index)).expect("Can't get planet by JSON path")[0]
    }

    let mercury = get_planet_as_json(&result.data, 0);
    check_planet(mercury, 1, "Mercury", "TERRESTRIAL_PLANET", "2439.7");

    let earth = get_planet_as_json(&result.data, 2);
    check_planet(earth, 3, "Earth", "TERRESTRIAL_PLANET", "6371.0");

    let neptune = get_planet_as_json(&result.data, 7);
    check_planet(neptune, 8, "Neptune", "ICE_GIANT", "24622.0");
}

fn check_planet(planet_json: &Value, id: i32, name: &str, planet_type: &str, mean_radius: &str) {
    fn check_property(planet_json: &Value, property_name: &str, property_expected_value: &str) {
        let json_path = format!("$..{}", property_name);
        assert_eq!(property_expected_value, jsonpath::select(&planet_json, &json_path).expect("Can't get property")[0].as_str().expect("Can't get property as str"));
    }
    check_property(planet_json, "id", &id.to_string());
    check_property(planet_json, "name", name);
    check_property(planet_json, "type", planet_type);
    check_property(planet_json, "details.meanRadius", mean_radius);
}

#[derive(Serialize)]
struct GraphQLCustomRequest {
    query: String,
    // variables: Option<Map<String, String>>,
    operation_name: String,
}

#[derive(Deserialize)]
struct GraphQLCustomResponse {
    data: serde_json::Value,
}
