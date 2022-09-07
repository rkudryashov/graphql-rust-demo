//! DISCLAIMER:
//! This is an example for only illustrative purposes. It demonstrates how to perform JWT verification via a router plugin

use std::convert::TryInto;
use std::ops::ControlFlow;

use apollo_router::{
    graphql,
    layers::ServiceBuilderExt,
    plugin::{Plugin, PluginInit},
    register_plugin,
    services::supergraph,
};
use http::StatusCode;
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use schemars::JsonSchema;
use serde::Deserialize;
use tower::{BoxError, ServiceBuilder, ServiceExt};
use tracing::debug;

use common_utils::Claims;

const ROLE_CONTEXT_PARAM_NAME: &str = "user_role";
const AUTHORIZATION_HEADER_NAME: &str = "authorization";
const ROLE_HEADER_NAME: &str = "role";

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

        let handler = move |request: supergraph::Request| {
            let headers = request.supergraph_request.headers();
            let maybe_auth_header_value = headers.get(AUTHORIZATION_HEADER_NAME);

            let jwt = match maybe_auth_header_value {
                Some(auth_header_value) => {
                    let auth_header_value_str = auth_header_value.to_str()?;
                    get_jwt_from_header_value(auth_header_value_str).to_string()
                }
                None => return Ok(ControlFlow::Continue(request)),
            };

            match decode_jwt(&jwt, &jwt_secret_key) {
                Ok(token_data) => {
                    let role = token_data.claims.role;
                    debug!("User role is: {}", &role);
                    request.context.insert(ROLE_CONTEXT_PARAM_NAME, role)?;

                    Ok(ControlFlow::Continue(request))
                }
                Err(e) => {
                    let error_message = format!("JWT is invalid: {}", e);
                    let response = supergraph::Response::error_builder()
                        .error(graphql::Error::builder().message(error_message).build())
                        .status_code(StatusCode::BAD_REQUEST)
                        .context(request.context)
                        .build()?;

                    Ok(ControlFlow::Break(response))
                }
            }
        };

        let request_mapper = |mut request: supergraph::Request| {
            let maybe_user_role: Option<String> = request
                .context
                .get(ROLE_CONTEXT_PARAM_NAME)
                .expect("This should not return an error");

            if let Some(user_role) = maybe_user_role {
                request.supergraph_request.headers_mut().insert(
                    ROLE_HEADER_NAME,
                    user_role
                        .try_into()
                        .expect("Role should always be converted to HeaderValue"),
                );
            }

            request
        };

        ServiceBuilder::new()
            .checkpoint(handler)
            .map_request(request_mapper)
            .service(service)
            .boxed()
    }
}

fn get_jwt_from_header_value(auth_header_value: &str) -> &str {
    let jwt_start_index = "Bearer ".len();
    &auth_header_value[jwt_start_index..auth_header_value.len()]
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
            .build()
            .await
            .unwrap();
    }
}
