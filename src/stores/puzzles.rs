use serde::Serialize;
use sqlx::{QueryBuilder, Sqlite, SqliteExecutor};

#[derive(Serialize)]
pub struct Puzzle {
    id: i64,
    #[serde(rename = "puzzleId")]
    puzzle_id: String,
    fen: String,
    moves: String,
    rating: i64,
    themes: String,
}

pub(crate) struct NewPuzzle {
    pub(crate) puzzle_id: String,
    pub(crate) fen: String,
    pub(crate) moves: String,
    pub(crate) rating: i32,
    pub(crate) themes: String,
}

pub(crate) async fn count<'e>(executor: impl SqliteExecutor<'e>) -> sqlx::Result<i64> {
    sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!: i64" FROM "Puzzle""#)
        .fetch_one(executor)
        .await
}

pub(crate) async fn find_by_id<'e>(
    executor: impl SqliteExecutor<'e>,
    id: i64,
) -> sqlx::Result<Option<Puzzle>> {
    sqlx::query_as!(
        Puzzle,
        r#"SELECT "id", "puzzleId" AS puzzle_id, "fen", "moves", "rating", "themes"
           FROM "Puzzle" WHERE "id" = ?"#,
        id,
    )
    .fetch_optional(executor)
    .await
}

pub(crate) async fn random_in_range<'e>(
    executor: impl SqliteExecutor<'e>,
    min: i32,
    max: Option<i32>,
) -> sqlx::Result<Option<Puzzle>> {
    match max {
        Some(max) => {
            sqlx::query_as!(
                Puzzle,
                r#"SELECT "id", "puzzleId" AS puzzle_id, "fen", "moves", "rating", "themes"
               FROM "Puzzle" WHERE "rating" >= ? AND "rating" < ? ORDER BY RANDOM() LIMIT 1"#,
                min,
                max,
            )
            .fetch_optional(executor)
            .await
        }
        None => {
            sqlx::query_as!(
                Puzzle,
                r#"SELECT "id", "puzzleId" AS puzzle_id, "fen", "moves", "rating", "themes"
               FROM "Puzzle" WHERE "rating" >= ? ORDER BY RANDOM() LIMIT 1"#,
                min,
            )
            .fetch_optional(executor)
            .await
        }
    }
}

pub(crate) async fn has_puzzles<'e>(executor: impl SqliteExecutor<'e>) -> sqlx::Result<bool> {
    sqlx::query_scalar!(r#"SELECT EXISTS (SELECT 1 FROM "Puzzle") AS "exists!: bool""#)
        .fetch_one(executor)
        .await
}

pub(crate) async fn clear<'e>(executor: impl SqliteExecutor<'e>) -> sqlx::Result<()> {
    sqlx::raw_sql(
        r#"DELETE FROM "Puzzle";
           DELETE FROM sqlite_sequence WHERE name = 'Puzzle';"#,
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub(crate) async fn insert_batch<'e>(
    executor: impl SqliteExecutor<'e>,
    batch: &[NewPuzzle],
) -> sqlx::Result<()> {
    let mut query = QueryBuilder::<Sqlite>::new(
        r#"INSERT INTO "Puzzle" ("puzzleId", "fen", "moves", "rating", "themes") "#,
    );
    query.push_values(batch, |mut row, puzzle| {
        row.push_bind(&puzzle.puzzle_id)
            .push_bind(&puzzle.fen)
            .push_bind(&puzzle.moves)
            .push_bind(puzzle.rating)
            .push_bind(&puzzle.themes);
    });
    query.build().execute(executor).await?;
    Ok(())
}
