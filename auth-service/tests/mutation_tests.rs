use std::str;

use actix_web::{App, guard, test, web};
use jsonpath_lib as jsonpath;
use serde::{Deserialize, Serialize};

use auth_service::{create_schema, index, prepare_env};
use auth_service::utils::Claims;

#[actix_rt::test]
async fn test_sign_in() {
    let pool = prepare_env();
    let schema = create_schema(pool);

    let mut service = test::init_service(App::new()
        .data(schema.clone())
        .service(web::resource("/").guard(guard::Post()).to(index)))
        .await;

    let mutation = r#"
        mutation {
            signIn (signInData: {username: "john_doe", password: "password"})
        }
        "#.to_string();

    let request_body = GraphQLCustomRequest {
        query: mutation,
    };

    let request = test::TestRequest::post().uri("/").set_json(&request_body).to_request();

    let result: GraphQLCustomResponse = test::read_response_json(&mut service, request).await;

    let jwt = jsonpath::select(&result.data, "$..signIn").expect("Can't get JWT")[0].as_str().expect("Can't get JWT string");

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
    let claims: Claims = serde_json::from_str(decoded_payload_string).expect("Can't deserialize claims");
    assert_eq!("john_doe", &claims.sub);
    assert_eq!("user", &claims.role);
}

#[actix_rt::test]
#[should_panic(expected = "Can't authenticate a user")]
async fn test_sign_in_fails() {
    let pool = prepare_env();
    let schema = create_schema(pool);

    let mut service = test::init_service(App::new()
        .data(schema.clone())
        .service(web::resource("/").guard(guard::Post()).to(index)))
        .await;

    let mutation = r#"
        mutation {
            signIn (signInData: {username: "john_doe", password: "wrong_password"})
        }
        "#.to_string();

    let request_body = GraphQLCustomRequest {
        query: mutation,
    };

    let request = test::TestRequest::post().uri("/").set_json(&request_body).to_request();

    test::call_service(&mut service, request).await;
}

#[derive(Serialize)]
struct GraphQLCustomRequest {
    query: String,
}

#[derive(Deserialize)]
struct GraphQLCustomResponse {
    data: serde_json::Value,
}
