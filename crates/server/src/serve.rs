//! `iasion serve`: the app on its listen address, the health check on its own
//! (ADR-0040).

use std::io::IsTerminal;
use std::sync::Arc;

use iasion_api::access::{Keys, REFRESH_INTERVAL};
use tokio::net::TcpListener;
use tokio::signal::unix::{SignalKind, signal};
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::database::{self, DatabaseStatus};
use crate::keys::{HttpKeySource, ProcessClock};
use crate::{health, notify};

pub async fn run() -> Result<(), String> {
    // Logs go to stdout, and from there to journald (ADR-0040, decision 16).
    tracing_subscriber::fmt()
        .with_ansi(std::io::stdout().is_terminal())
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = Config::from_env()?;
    let pool = database::connect_app().await?;
    database::ensure_migrated(&pool).await?;

    let source = Arc::new(HttpKeySource::new(config.certs_url)?);
    let keys = Arc::new(Keys::new(source, Arc::new(ProcessClock::start())));
    // Without keys the app still starts and refuses requests with 503 until a fetch
    // succeeds (ADR-0037, decision 6).
    if let Err(error) = keys.refresh().await {
        tracing::warn!(%error, "no Access keys at startup");
    }
    tokio::spawn(refresh_hourly(keys.clone()));

    let app = iasion_api::router(config.api, keys, Arc::new(DatabaseStatus(pool.clone())));
    let app_listener = bind(config.listen_addr).await?;
    let health_listener = bind(config.health_addr).await?;
    tracing::info!(app = %config.listen_addr, health = %config.health_addr, "listening");
    notify::ready()?;

    let health = axum::serve(health_listener, health::router(pool));
    tokio::spawn(async move {
        if let Err(error) = health.await {
            tracing::error!(%error, "the health listener stopped");
        }
    });
    axum::serve(app_listener, app)
        .with_graceful_shutdown(shutdown())
        .await
        .map_err(|error| format!("serve: {error}"))
}

async fn bind(address: std::net::SocketAddr) -> Result<TcpListener, String> {
    TcpListener::bind(address)
        .await
        .map_err(|error| format!("cannot listen on {address}: {error}"))
}

async fn refresh_hourly(keys: Arc<Keys>) {
    let mut interval = tokio::time::interval(REFRESH_INTERVAL);
    interval.tick().await;
    loop {
        interval.tick().await;
        if let Err(error) = keys.refresh().await {
            tracing::warn!(%error, "refreshing the Access keys failed");
        }
    }
}

/// Resolves on SIGTERM (systemd, Podman) or SIGINT (a terminal).
async fn shutdown() {
    let Ok(mut terminate) = signal(SignalKind::terminate()) else {
        let _ = tokio::signal::ctrl_c().await;
        return;
    };
    tokio::select! {
        _ = terminate.recv() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
    tracing::info!("shutting down");
}
