use anyhow::{Context, Result, ensure};
use sqlx::migrate::Migrator;
use sqlx::{SqliteConnection, SqlitePool};

static MIGRATOR: Migrator = sqlx::migrate!("src/db/migrations");

pub(crate) async fn migrate(pool: &SqlitePool) -> Result<()> {
    // SQLite's SQLx migrator has no advisory lock. Reserve the write lock before reading history.
    let mut transaction = pool.begin_with("BEGIN IMMEDIATE").await?;

    let adopt_prisma: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = '_prisma_migrations')
         AND NOT EXISTS (SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = '_sqlx_migrations')",
    )
    .fetch_one(&mut *transaction)
    .await?;

    if adopt_prisma && let Some(version) = prisma_version(&mut transaction).await? {
        ensure!(
            MIGRATOR.version_exists(version),
            "unknown existing migration version {version}"
        );
        MIGRATOR.skip(&mut *transaction, Some(version)).await?;
    }

    MIGRATOR.run(&mut *transaction).await?;
    transaction.commit().await?;
    Ok(())
}

async fn prisma_version(connection: &mut SqliteConnection) -> Result<Option<i64>> {
    let incomplete: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM _prisma_migrations
         WHERE finished_at IS NULL AND rolled_back_at IS NULL)",
    )
    .fetch_one(&mut *connection)
    .await?;
    ensure!(
        !incomplete,
        "existing Prisma migration history is incomplete"
    );

    let name: Option<String> = sqlx::query_scalar(
        "SELECT migration_name FROM _prisma_migrations
         WHERE finished_at IS NOT NULL AND rolled_back_at IS NULL
         ORDER BY migration_name DESC LIMIT 1",
    )
    .fetch_optional(connection)
    .await?;
    let Some(name) = name else {
        return Ok(None);
    };

    let (version, _) = name
        .split_once('_')
        .context("invalid Prisma migration name")?;
    Ok(Some(
        version
            .parse()
            .context("invalid Prisma migration version")?,
    ))
}
