//! DISCLAIMER:
//! This is an example for only illustrative purposes. It demonstrates how to perform JWT verification via a router plugin

use std::convert::TryInto;
use std::ops::ControlFlow;

use apollo_router::{
    graphql,
    layers::ServiceBuilderExt,
    plugin::Plugin,
    register_plugin,
    services::{RouterRequest, RouterResponse},
};
use futures::stream::BoxStream;
use http::StatusCode;
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use schemars::JsonSchema;
use serde::Deserialize;
use tower::util::BoxService;
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

    async fn new(configuration: Self::Config) -> Result<Self, BoxError> {
        Ok(JwtValidation {
            secret_key: configuration.secret_key,
        })
    }

    fn router_service(
        &mut self,
        service: BoxService<
            RouterRequest,
            RouterResponse<BoxStream<'static, graphql::Response>>,
            BoxError,
        >,
    ) -> BoxService<RouterRequest, RouterResponse<BoxStream<'static, graphql::Response>>, BoxError>
    {
        let jwt_secret_key = self.secret_key.clone();
        ServiceBuilder::new()
            .checkpoint(move |mut request: RouterRequest| {
                let headers = request.originating_request.headers_mut();
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
                        let response = RouterResponse::error_builder()
                            .error(apollo_router::graphql::Error {
                                message: format!("JWT is invalid: {}", e),
                                ..Default::default()
                            })
                            .status_code(StatusCode::BAD_REQUEST)
                            .context(request.context)
                            .build()?;

                        Ok(ControlFlow::Break(response.boxed()))
                    }
                }
            })
            .map_request(|mut request: RouterRequest| {
                let maybe_user_role: Option<String> = request
                    .context
                    .get(ROLE_CONTEXT_PARAM_NAME)
                    .expect("This should not return an error");

                if let Some(user_role) = maybe_user_role {
                    request.originating_request.headers_mut().insert(
                        ROLE_HEADER_NAME,
                        user_role
                            .try_into()
                            .expect("Role should always be converted to HeaderValue"),
                    );
                }

                request
            })
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
        apollo_router::plugin::plugins()
            .get("demo.jwt_validation")
            .expect("Plugin not found")
            .create_instance(&serde_json::json!({"secret_key" : "example"}))
            .await
            .unwrap();
    }
}
