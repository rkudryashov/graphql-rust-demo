use actix_web::{App, guard, test, web};
use chrono::NaiveDate;
use jsonpath_lib as jsonpath;
use serde::{Deserialize, Serialize};
use serde_json::Map;

use satellites_service::{create_schema, index, prepare_env};

const TEST_FIELDS_FRAGMENT: &str = "
    fragment testFields on Satellite {
        id
        name
        firstSpacecraftLandingDate
    }
";

#[actix_rt::test]
async fn test_satellites() {
    let pool = prepare_env();
    let schema = create_schema(pool);

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
        operation_name: "query".to_string(),
    };

    let request = test::TestRequest::post().uri("/").set_json(&request_body).to_request();

    let result: GraphQLCustomResponse = test::read_response_json(&mut service, request).await;

    fn get_satellite_as_json(all_satellites: &serde_json::Value, index: i32) -> &serde_json::Value {
        jsonpath::select(all_satellites, &format!("$..satellites[{}]", index)).expect("Can't get satellite by JSON path")[0]
    }

    let moon_json = get_satellite_as_json(&result.data, 0);
    check_satellite(moon_json, "Moon", Some(NaiveDate::from_ymd(1959, 9, 13)));

    let titan_json = get_satellite_as_json(&result.data, 7);
    check_satellite(titan_json, "Titan", None);
}

#[actix_rt::test]
async fn test_satellite() {
    let pool = prepare_env();
    let schema = create_schema(pool);

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
        operation_name: "query".to_string(),
    };

    let request = test::TestRequest::post().uri("/").set_json(&request_body).to_request();

    let result: GraphQLCustomResponse = test::read_response_json(&mut service, request).await;

    let moon_json = jsonpath::select(&result.data, "$..satellite").expect("Can't get satellite by JSON path")[0];
    check_satellite(moon_json, "Moon", Some(NaiveDate::from_ymd(1959, 9, 13)));
}

#[actix_rt::test]
#[should_panic(expected = "life_exists can only be accessed by authenticated user with `admin` role")]
async fn test_satellite_should_panic() {
    let pool = prepare_env();
    let schema = create_schema(pool);

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
        operation_name: "query".to_string(),
    };

    let request = test::TestRequest::post().uri("/").set_json(&request_body).to_request();

    test::call_service(&mut service, request).await;
}

fn check_satellite(satellite_json: &serde_json::Value, name: &str, first_spacecraft_landing_date: Option<NaiveDate>) {
    assert_eq!(name, jsonpath::select(&satellite_json, "$..name").expect("Can't get property")[0].as_str().expect("Can't get property as str"));
    match first_spacecraft_landing_date {
        Some(date) => {
            let date_string = jsonpath::select(&satellite_json, "$..firstSpacecraftLandingDate").expect("Can't get property")[0].as_str().expect("Can't get property as str");
            assert_eq!(date, date_string.parse::<NaiveDate>().expect("Can't parse &str"));
        }
        None => {
            assert!(jsonpath::select(&satellite_json, "$..firstSpacecraftLandingDate").expect("Can't get property")[0].is_null());
        }
    };
}

#[derive(Serialize)]
struct GraphQLCustomRequest {
    query: String,
    variables: Map<String, serde_json::Value>,
    operation_name: String,
}

#[derive(Deserialize)]
struct GraphQLCustomResponse {
    data: serde_json::Value,
}
