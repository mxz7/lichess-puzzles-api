use std::path::Path;

use anyhow::{Context, Result, ensure};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

pub(crate) async fn open() -> Result<SqlitePool> {
    let url = std::env::var("DATABASE_URL").context("DATABASE_URL is required")?;
    let filename = url.strip_prefix("file:").unwrap_or(&url);
    ensure!(
        !filename.is_empty(),
        "DATABASE_URL must name a SQLite database"
    );

    let options = SqliteConnectOptions::new()
        .filename(Path::new(filename))
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .connect_with(options)
        .await
        .with_context(|| format!("could not open SQLite database {filename}"))?;
    super::migrate::migrate(&pool)
        .await
        .with_context(|| format!("could not migrate SQLite database {filename}"))?;
    Ok(pool)
}
