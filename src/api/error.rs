use axum::Json;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Clone)]
pub(super) struct ApiError(pub(super) String);

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        Self(error.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        eprintln!("{}", self.0);
        let mut response = text(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error");
        response.extensions_mut().insert(self);
        response
    }
}

pub(super) async fn not_found() -> Response {
    text(StatusCode::NOT_FOUND, "404 Not Found")
}

fn text(status: StatusCode, body: &'static str) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, "text/plain; charset=UTF-8")],
        body,
    )
        .into_response()
}

pub(super) fn json(status: StatusCode, error: &str) -> Response {
    #[derive(Serialize)]
    struct Error<'a> {
        error: &'a str,
        status: u16,
    }

    (
        status,
        Json(Error {
            error,
            status: status.as_u16(),
        }),
    )
        .into_response()
}
