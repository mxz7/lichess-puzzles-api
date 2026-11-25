use axum::body::{Body, to_bytes};
use axum::extract::Request;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::app::AppState;

use super::router;

#[tokio::test]
async fn preserves_legacy_puzzle_routes_and_difficulty_boundaries() -> anyhow::Result<()> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    crate::db::migrate(&pool).await?;
    let state = AppState::new(pool.clone());
    assert_eq!(state.puzzle_count().await?, 0);
    sqlx::query!(
        r#"INSERT INTO "Puzzle" ("puzzleId", "fen", "moves", "rating", "themes")
           VALUES ('gap', 'fen', 'moves', 2499, 'themes'),
                  ('top', 'fen', 'moves', 2500, 'themes')"#,
    )
    .execute(&pool)
    .await?;

    let app = router(state.clone());
    for (method, uri, status) in [
        ("GET", "/", 200),
        ("POST", "/", 404),
        ("GET", "/puzzles/top", 404),
        ("GET", "/puzzles/{id}", 500),
        ("GET", "/puzzles/random", 200),
        ("GET", "/puzzles/random/", 404),
        ("GET", "/puzzles/random/expert", 404),
        ("GET", "/puzzles/random/unknown", 400),
        ("GET", "/puzzles/random/GRANDMASTER", 200),
        ("HEAD", "/puzzles/random/grandmaster", 200),
    ] {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())?;
        let response = app.clone().oneshot(request).await?;
        assert_eq!(response.status(), status, "{method} {uri}");
        let body = to_bytes(response.into_body(), 1024).await?;
        if method == "HEAD" {
            assert!(body.is_empty());
        } else if uri == "/puzzles/random/GRANDMASTER" {
            let puzzle: serde_json::Value = serde_json::from_slice(&body)?;
            assert_eq!(puzzle["puzzleId"], "top");
            assert_eq!(puzzle["rating"], 2500);
        }
    }

    sqlx::query!(r#"DELETE FROM "Puzzle""#)
        .execute(&pool)
        .await?;
    assert_eq!(state.puzzle_count().await?, 2);

    Ok(())
}
