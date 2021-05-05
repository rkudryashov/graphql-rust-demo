use std::str::FromStr;

use actix_web::{test, App};
use chrono::NaiveDate;
use jsonpath_lib as jsonpath;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use testcontainers::clients::Cli;

use satellites_service::graphql::LifeExists::{NoData, OpenQuestion};
use satellites_service::{configure_service, create_schema_with_context, graphql::LifeExists};

mod common;

const TEST_FIELDS_FRAGMENT: &str = "
    fragment testFields on Satellite {
        id
        name
        firstSpacecraftLandingDate
        lifeExists
    }
";

#[actix_rt::test]
async fn test_get_satellites() {
    let docker = Cli::default();
    let (_pg_container, pool) = common::setup(&docker);

    let mut service = test::init_service(
        App::new()
            .configure(configure_service)
            .data(create_schema_with_context(pool)),
    )
    .await;

    let query = "
        {
            getSatellites {
                ... testFields
            }
        }
        "
    .to_string()
        + TEST_FIELDS_FRAGMENT;

    let request_body = GraphQLCustomRequest {
        query,
        variables: Map::new(),
    };

    let request = test::TestRequest::post()
        .uri("/")
        .set_json(&request_body)
        .to_request();

    let response: GraphQLCustomResponse = test::read_response_json(&mut service, request).await;

    let response_data = response.data.expect("Response doesn't contain data");

    fn get_satellite_as_json(all_satellites: &serde_json::Value, index: i32) -> &serde_json::Value {
        jsonpath::select(all_satellites, &format!("$.getSatellites[{}]", index))
            .expect("Can't get satellite by JSON path")[0]
    }

    let moon_json = get_satellite_as_json(&response_data, 0);
    check_satellite(
        moon_json,
        "Moon",
        Some(NaiveDate::from_ymd(1959, 9, 13)),
        OpenQuestion,
    );

    let titan_json = get_satellite_as_json(&response_data, 7);
    check_satellite(titan_json, "Titan", None, NoData);
}

#[actix_rt::test]
async fn test_get_satellite() {
    let docker = Cli::default();
    let (_pg_container, pool) = common::setup(&docker);

    let mut service = test::init_service(
        App::new()
            .configure(configure_service)
            .data(create_schema_with_context(pool)),
    )
    .await;

    let query = "
        {
            getSatellite(id: 1) {
                ... testFields
            }
        }
        "
    .to_string()
        + TEST_FIELDS_FRAGMENT;

    let request_body = GraphQLCustomRequest {
        query,
        variables: Map::new(),
    };

    let request = test::TestRequest::post()
        .uri("/")
        .set_json(&request_body)
        .to_request();

    let response: GraphQLCustomResponse = test::read_response_json(&mut service, request).await;

    let response_data = response.data.expect("Response doesn't contain data");

    let moon_json = jsonpath::select(&response_data, "$.getSatellite")
        .expect("Can't get satellite by JSON path")[0];
    check_satellite(
        moon_json,
        "Moon",
        Some(NaiveDate::from_ymd(1959, 9, 13)),
        OpenQuestion,
    );
}

fn check_satellite(
    satellite_json: &serde_json::Value,
    name: &str,
    first_spacecraft_landing_date: Option<NaiveDate>,
    life_exists: LifeExists,
) {
    let json_name = jsonpath::select(&satellite_json, "$.name").expect("Can't get property")[0]
        .as_str()
        .expect("Can't get property as str");
    assert_eq!(name, json_name);

    let json_life_exists = LifeExists::from_str(
        jsonpath::select(&satellite_json, "$.lifeExists").expect("Can't get property")[0]
            .as_str()
            .expect("Can't get property as str"),
    )
    .expect("Can't convert &str to LifeExists");
    assert_eq!(life_exists, json_life_exists);

    match first_spacecraft_landing_date {
        Some(date) => {
            let date_string = jsonpath::select(&satellite_json, "$.firstSpacecraftLandingDate")
                .expect("Can't get property")[0]
                .as_str()
                .expect("Can't get property as str");
            assert_eq!(
                date,
                date_string.parse::<NaiveDate>().expect("Can't parse str")
            );
        }
        None => {
            assert!(
                jsonpath::select(&satellite_json, "$.firstSpacecraftLandingDate")
                    .expect("Can't get property")[0]
                    .is_null()
            );
        }
    };
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
