use std::str;

use actix_web::{test, web, App};
use jsonpath_lib as jsonpath;
use serde::{Deserialize, Serialize};
use testcontainers::clients::Cli;

use auth_service::{configure_service, create_schema_with_context};
use common_utils::Claims;

mod common;

#[actix_rt::test]
async fn test_sign_in() {
    let docker = Cli::default();
    let (_pg_container, pool) = common::setup(&docker);

    let service = test::init_service(
        App::new()
            .configure(configure_service)
            .app_data(web::Data::new(create_schema_with_context(pool))),
    )
    .await;

    let mutation = r#"
        mutation {
            signIn(input: { username: "john_doe", password: "password" })
        }
        "#
    .to_string();

    let request_body = GraphQLCustomRequest { query: mutation };

    let request = test::TestRequest::post()
        .uri("/")
        .set_json(&request_body)
        .to_request();

    let response: GraphQLCustomResponse = test::call_and_read_body_json(&service, request).await;

    let jwt = jsonpath::select(
        &response.data.expect("Response doesn't contain data"),
        "$.signIn",
    )
    .expect("Can't get JWT by path")
    .first()
    .expect("Can't get JWT")
    .as_str()
    .expect("Can't get JWT string")
    .to_string();

    let first_dot_index = jwt.find('.').expect("Incorrect JWT");
    let last_dot_index = jwt.rfind('.').expect("Incorrect JWT");

    let encoded_header = &jwt[0..first_dot_index];
    let decoded_header = base64::decode(encoded_header).expect("Can't decode Base64");
    let decoded_header_string = str::from_utf8(&decoded_header).expect("Can't convert to str");
    let expected_header = "{\"typ\":\"JWT\",\"alg\":\"HS256\"}";
    assert_eq!(expected_header, decoded_header_string);

    let encoded_payload = &jwt[(first_dot_index + 1)..last_dot_index];
    let decoded_payload = base64::decode(encoded_payload).expect("Can't decode Base64");
    let decoded_payload_string = str::from_utf8(&decoded_payload).expect("Can't convert to str");
    let claims: Claims =
        serde_json::from_str(decoded_payload_string).expect("Can't deserialize claims");
    assert_eq!("john_doe", &claims.sub);
    assert_eq!("ADMIN", &claims.role);
}

#[actix_rt::test]
async fn test_sign_in_fails() {
    let docker = Cli::default();
    let (_pg_container, pool) = common::setup(&docker);

    let service = test::init_service(
        App::new()
            .configure(configure_service)
            .app_data(web::Data::new(create_schema_with_context(pool))),
    )
    .await;

    let mutation = r#"
        mutation {
            signIn(input: { username: "john_doe", password: "wrong_password" })
        }
        "#
    .to_string();

    let request_body = GraphQLCustomRequest { query: mutation };

    let request = test::TestRequest::post()
        .uri("/")
        .set_json(&request_body)
        .to_request();

    let response: GraphQLCustomResponse = test::call_and_read_body_json(&service, request).await;

    let error_message = jsonpath::select(
        &response.errors.expect("Response doesn't contain errors"),
        "$[0].message",
    )
    .expect("Can't get error message by path")
    .first()
    .expect("Can't get error message")
    .as_str()
    .expect("Can't get error message")
    .to_string();

    assert_eq!("Can't authenticate a user", error_message);
}

#[derive(Serialize)]
struct GraphQLCustomRequest {
    query: String,
}

#[derive(Deserialize)]
struct GraphQLCustomResponse {
    data: Option<serde_json::Value>,
    errors: Option<serde_json::Value>,
}
