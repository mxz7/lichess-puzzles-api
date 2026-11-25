mod error;
mod http;

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum::{Router, routing::get};

use crate::app::AppState;
use crate::request_log::Logger;

use self::error::ApiError;

pub(crate) fn router(app: AppState) -> Router {
    Router::new()
        .route("/", get(http::health::get))
        .route("/puzzles/random", get(http::puzzles::random))
        .route(
            "/puzzles/random/{difficulty}",
            get(http::puzzles::by_difficulty),
        )
        // The original route matched the literal braces and returned an error.
        .route("/puzzles/{{id}}", get(http::puzzles::missing_id))
        .fallback(error::not_found)
        .method_not_allowed_fallback(error::not_found)
        .with_state(app)
}

pub(crate) async fn log_request(
    State(logger): State<Arc<Logger>>,
    request: Request,
    next: Next,
) -> Response {
    let log = logger.start(&request, request.uri().path());
    let mut response = next.run(request).await;
    let error = response.extensions_mut().remove::<ApiError>();
    logger.finish(log, response.status(), error.map(|error| error.0));
    response
}

#[cfg(test)]
mod tests;
