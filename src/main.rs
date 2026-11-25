mod api;
mod app;
mod db;
mod import;
mod request_log;
mod stores;

use std::sync::Arc;

use anyhow::{Context, Result, bail};
use axum::middleware;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => serve().await,
        [command] if command == "migrate" => db::open().await.map(drop),
        [command] if command == "import-puzzles" => {
            let pool = db::open().await?;
            import::puzzles(&pool, false)
                .await
                .context("Puzzle import failed")
        }
        _ => bail!("Usage: lichess-puzzles [migrate | import-puzzles]"),
    }
}

async fn serve() -> Result<()> {
    let pool = db::open().await?;
    let (logger, log_worker) = request_log::Logger::new()?;
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server is running on http://localhost:3000");

    let app = api::router(app::AppState::new(pool.clone())).layer(middleware::from_fn_with_state(
        Arc::new(logger),
        api::log_request,
    ));
    let server = async {
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .context("HTTP server failed")
    };
    let import = async {
        import::puzzles(&pool, true)
            .await
            .context("Puzzle import failed")
    };
    let result = tokio::try_join!(server, import);

    if let Some(worker) = log_worker {
        worker.await.context("Loki log worker failed")?;
    }
    result?;
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut terminate = signal(SignalKind::terminate()).expect("register SIGTERM handler");
        tokio::select! {
            _ = terminate.recv() => {},
            _ = tokio::signal::ctrl_c() => {},
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}
