use std::convert::Infallible;

use async_graphql::{EmptyMutation, EmptySubscription, QueryBuilder, Schema};
use async_graphql::http::{GQLResponse, playground_source};
use async_graphql_warp::BadRequest;
use http::StatusCode;
use warp::{Filter, http::Response, Rejection, Reply};

use crate::graphql::Query;
use crate::model::Storage;

mod graphql;
mod model;

#[tokio::main]
async fn main() {
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(Storage::new())
        .finish();

    let graphql_post = async_graphql_warp::graphql(schema).and_then(
        |(schema, builder): (_, QueryBuilder)| async move {
            let resp = builder.execute(&schema).await;
            Ok::<_, Infallible>(warp::reply::json(&GQLResponse(resp)).into_response())
        },
    );

    let graphql_playground = warp::path::end().and(warp::get()).map(|| {
        Response::builder()
            .header("content-type", "text/html")
            .body(playground_source("/", None))
    });

    let routes = graphql_post
        .or(graphql_playground)
        .recover(|err: Rejection| async move {
            if let Some(BadRequest(err)) = err.find() {
                return Ok::<_, Infallible>(warp::reply::with_status(
                    err.to_string(),
                    StatusCode::BAD_REQUEST,
                ));
            }

            Ok(warp::reply::with_status(
                "INTERNAL_SERVER_ERROR".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        });

    warp::serve(routes).run(([0, 0, 0, 0], 8002)).await;
}
