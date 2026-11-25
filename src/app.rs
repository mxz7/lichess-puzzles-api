use std::sync::Arc;

use sqlx::SqlitePool;
use tokio::sync::Mutex;

use crate::stores;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db: SqlitePool,
    puzzle_count: Arc<Mutex<i64>>,
}

impl AppState {
    pub(crate) fn new(db: SqlitePool) -> Self {
        Self {
            db,
            puzzle_count: Arc::new(Mutex::new(0)),
        }
    }

    pub(crate) async fn puzzle_count(&self) -> sqlx::Result<i64> {
        let mut count = self.puzzle_count.lock().await;
        if *count == 0 {
            *count = stores::puzzles::count(&self.db).await?;
        }
        Ok(*count)
    }
}
