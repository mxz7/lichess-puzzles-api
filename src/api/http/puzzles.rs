use axum::Json;
use axum::extract::{Path, State, rejection::PathRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::api::error::{self, ApiError};
use crate::app::AppState;
use crate::stores;
use crate::stores::puzzles::Puzzle;

pub(in crate::api) async fn random(State(app): State<AppState>) -> Result<Response, ApiError> {
    let count = app.puzzle_count().await?;
    let puzzle = if count > 0 {
        let id = rand::random_range(1..=count);
        stores::puzzles::find_by_id(&app.db, id).await?
    } else {
        None
    };

    Ok(puzzle_response(puzzle, "No puzzles found"))
}

pub(in crate::api) async fn by_difficulty(
    State(app): State<AppState>,
    difficulty: Result<Path<String>, PathRejection>,
) -> Result<Response, ApiError> {
    let Ok(Path(difficulty)) = difficulty else {
        return Ok(invalid_difficulty());
    };

    let difficulty = difficulty.to_lowercase();
    let missing = format!("No puzzles found for difficulty '{difficulty}'");
    let (min, max) = match difficulty.as_str() {
        "beginner" => (0, Some(1000)),
        "easy" => (1000, Some(1300)),
        "medium" => (1300, Some(1600)),
        "hard" => (1600, Some(1900)),
        "expert" => (1900, Some(2499)),
        "grandmaster" => (2500, None),
        // The legacy JavaScript object recognized these inherited properties.
        "constructor" | "__proto__" => return Ok(puzzle_response(None, &missing)),
        _ => return Ok(invalid_difficulty()),
    };

    Ok(puzzle_response(
        stores::puzzles::random_in_range(&app.db, min, max).await?,
        &missing,
    ))
}

pub(in crate::api) async fn missing_id() -> ApiError {
    ApiError("Missing puzzle ID".to_owned())
}

fn puzzle_response(puzzle: Option<Puzzle>, missing: &str) -> Response {
    match puzzle {
        Some(puzzle) => Json(puzzle).into_response(),
        None => error::json(StatusCode::NOT_FOUND, missing),
    }
}

fn invalid_difficulty() -> Response {
    error::json(
        StatusCode::BAD_REQUEST,
        "Invalid difficulty. Use one of: beginner, easy, medium, hard, expert, grandmaster",
    )
}
