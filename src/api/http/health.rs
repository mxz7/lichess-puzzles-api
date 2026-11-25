use axum::{Json, response::IntoResponse};
use serde::Serialize;

pub(in crate::api) async fn get() -> impl IntoResponse {
    Json(Health {
        status: "ok",
        message: "powered by lichess!",
    })
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    message: &'static str,
}
