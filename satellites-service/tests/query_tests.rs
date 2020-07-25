use actix_web::{App, guard, test, web};
use chrono::NaiveDate;
use jsonpath_lib as jsonpath;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use testcontainers::clients::Cli;

use satellites_service::index;

mod common;

const TEST_FIELDS_FRAGMENT: &str = "
    fragment testFields on Satellite {
        id
        name
        firstSpacecraftLandingDate
    }
";

#[actix_rt::test]
async fn test_satellites() {
    let docker = Cli::default();
    let (schema, _pg_container) = common::setup(&docker);

    let mut service = test::init_service(App::new()
        .data(schema.clone())
        .service(web::resource("/").guard(guard::Post()).to(index)))
        .await;

    let query = "
        {
            satellites {
                ... testFields
            }
        }
        ".to_string() + TEST_FIELDS_FRAGMENT;

    let request_body = GraphQLCustomRequest {
        query,
        variables: Map::new(),
    };

    let request = test::TestRequest::post().uri("/").set_json(&request_body).to_request();

    let response: GraphQLCustomResponse = test::read_response_json(&mut service, request).await;

    let response_data = response.data.expect("Response doesn't contain data");

    fn get_satellite_as_json(all_satellites: &serde_json::Value, index: i32) -> &serde_json::Value {
        jsonpath::select(all_satellites, &format!("$.satellites[{}]", index)).expect("Can't get satellite by JSON path")[0]
    }

    let moon_json = get_satellite_as_json(&response_data, 0);
    check_satellite(moon_json, "Moon", Some(NaiveDate::from_ymd(1959, 9, 13)));

    let titan_json = get_satellite_as_json(&response_data, 7);
    check_satellite(titan_json, "Titan", None);
}

#[actix_rt::test]
async fn test_satellite() {
    let docker = Cli::default();
    let (schema, _pg_container) = common::setup(&docker);

    let mut service = test::init_service(App::new()
        .data(schema.clone())
        .service(web::resource("/").guard(guard::Post()).to(index)))
        .await;

    let query = "
        {
            satellite(id: 1) {
                ... testFields
            }
        }
        ".to_string() + TEST_FIELDS_FRAGMENT;

    let request_body = GraphQLCustomRequest {
        query,
        variables: Map::new(),
    };

    let request = test::TestRequest::post().uri("/").set_json(&request_body).to_request();

    let response: GraphQLCustomResponse = test::read_response_json(&mut service, request).await;

    let response_data = response.data.expect("Response doesn't contain data");

    let moon_json = jsonpath::select(&response_data, "$.satellite").expect("Can't get satellite by JSON path")[0];
    check_satellite(moon_json, "Moon", Some(NaiveDate::from_ymd(1959, 9, 13)));
}

#[actix_rt::test]
async fn test_satellite_should_return_forbidden() {
    let docker = Cli::default();
    let (schema, _pg_container) = common::setup(&docker);

    let mut service = test::init_service(App::new()
        .data(schema.clone())
        .service(web::resource("/").guard(guard::Post()).to(index)))
        .await;

    let query = "
        {
            satellite(id: 7) {
                ... testFields
                lifeExists
            }
        }
        ".to_string() + TEST_FIELDS_FRAGMENT;

    let request_body = GraphQLCustomRequest {
        query,
        variables: Map::new(),
    };

    let request = test::TestRequest::post().uri("/").set_json(&request_body).to_request();

    let response: GraphQLCustomResponse = test::read_response_json(&mut service, request).await;

    let errors = &response.errors.expect("Response doesn't contain errors");
    let error_message = jsonpath::select(errors, "$.[0].message").expect("Can't get error by JSON path")[0];
    assert_eq!("Forbidden", error_message);
}

fn check_satellite(satellite_json: &serde_json::Value, name: &str, first_spacecraft_landing_date: Option<NaiveDate>) {
    assert_eq!(name, jsonpath::select(&satellite_json, "$.name").expect("Can't get property")[0].as_str().expect("Can't get property as str"));
    match first_spacecraft_landing_date {
        Some(date) => {
            let date_string = jsonpath::select(&satellite_json, "$.firstSpacecraftLandingDate").expect("Can't get property")[0].as_str().expect("Can't get property as str");
            assert_eq!(date, date_string.parse::<NaiveDate>().expect("Can't parse str"));
        }
        None => {
            assert!(jsonpath::select(&satellite_json, "$.firstSpacecraftLandingDate").expect("Can't get property")[0].is_null());
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
    errors: Option<serde_json::Value>,
}
