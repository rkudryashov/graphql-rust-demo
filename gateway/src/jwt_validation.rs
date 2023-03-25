//! DISCLAIMER:
//! This is an example for only illustrative purposes. It demonstrates how to perform JWT verification via a router plugin

use std::ops::ControlFlow;

use apollo_router::{
    graphql,
    layers::ServiceBuilderExt,
    plugin::{Plugin, PluginInit},
    register_plugin,
    services::supergraph,
    Context,
};
use http::header::AUTHORIZATION;
use http::StatusCode;
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use schemars::JsonSchema;
use serde::Deserialize;
use tower::{BoxError, ServiceBuilder, ServiceExt};
use tracing::debug;

use common_utils::Claims;

const ROLE_CONTEXT_PARAM_NAME: &str = "user_role";

#[derive(Deserialize, JsonSchema)]
struct JwtValidationConfig {
    secret_key: String,
}

#[derive(Debug)]
struct JwtValidation {
    secret_key: String,
}

#[async_trait::async_trait]
impl Plugin for JwtValidation {
    type Config = JwtValidationConfig;

    async fn new(init: PluginInit<Self::Config>) -> Result<Self, BoxError> {
        Ok(JwtValidation {
            secret_key: init.config.secret_key,
        })
    }

    fn supergraph_service(&self, service: supergraph::BoxService) -> supergraph::BoxService {
        let jwt_secret_key = self.secret_key.clone();

        fn failure_message(
            context: Context,
            message: String,
            status: StatusCode,
        ) -> Result<ControlFlow<supergraph::Response, supergraph::Request>, BoxError> {
            let response = supergraph::Response::error_builder()
                .error(
                    graphql::Error::builder()
                        .message(message)
                        .extension_code("AUTH_ERROR")
                        .build(),
                )
                .status_code(status)
                .context(context)
                .build()?;
            Ok(ControlFlow::Break(response))
        }

        let handler = move |request: supergraph::Request| {
            let auth_header_value_result =
                match request.supergraph_request.headers().get(AUTHORIZATION) {
                    Some(auth_header_value) => auth_header_value.to_str(),
                    // in this project, I decided to allow the passage of requests without the AUTHORIZATION header. then each subgraph performs authorization based on the ROLE header
                    // your case may be different. be careful
                    None => return Ok(ControlFlow::Continue(request)),
                };

            let auth_header_value_untrimmed = match auth_header_value_result {
                Ok(auth_header_value) => auth_header_value,
                Err(_not_a_string_error) => {
                    return failure_message(
                        request.context,
                        "'AUTHORIZATION' header is not convertible to a string".to_string(),
                        StatusCode::BAD_REQUEST,
                    )
                }
            };

            let auth_header_value = auth_header_value_untrimmed.trim();

            if !auth_header_value
                .to_uppercase()
                .as_str()
                .starts_with("BEARER ")
            {
                return failure_message(
                    request.context,
                    format!("'{auth_header_value_untrimmed}' is not correctly formatted"),
                    StatusCode::BAD_REQUEST,
                );
            }

            let auth_header_value_parts: Vec<&str> = auth_header_value.splitn(2, ' ').collect();
            if auth_header_value_parts.len() != 2 {
                return failure_message(
                    request.context,
                    format!("'{auth_header_value}' is not correctly formatted"),
                    StatusCode::BAD_REQUEST,
                );
            }

            let jwt = auth_header_value_parts[1].trim_end();

            match decode_jwt(&jwt, &jwt_secret_key) {
                Ok(token_data) => {
                    let role = token_data.claims.role;
                    debug!("User role is: {}", &role);
                    if let Err(error) = request.context.insert(ROLE_CONTEXT_PARAM_NAME, role) {
                        return failure_message(
                            request.context,
                            format!("Failed to pass a user's role: {}", error),
                            StatusCode::INTERNAL_SERVER_ERROR,
                        );
                    }

                    Ok(ControlFlow::Continue(request))
                }
                Err(e) => {
                    let error_message = format!("JWT is invalid: {}", e);
                    return failure_message(
                        request.context,
                        error_message,
                        StatusCode::BAD_REQUEST,
                    );
                }
            }
        };

        ServiceBuilder::new()
            .checkpoint(handler)
            .service(service)
            .boxed()
    }
}

// this also includes JWT validation
fn decode_jwt(jwt: &str, secret_key: &str) -> jsonwebtoken::errors::Result<TokenData<Claims>> {
    decode::<Claims>(
        jwt,
        &DecodingKey::from_secret(secret_key.as_bytes()),
        &Validation::default(),
    )
}

register_plugin!("demo", "jwt_validation", JwtValidation);

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn plugin_registered() {
        let config = serde_json::json!({
            "plugins": {
                "demo.jwt_validation": {
                    "secret_key": "example"
                }
            }
        });

        apollo_router::TestHarness::builder()
            .configuration_json(config)
            .unwrap()
            .build_supergraph()
            .await
            .unwrap();
    }
}
