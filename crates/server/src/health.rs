//! The local health check (ADR-0040, decision 16): `serve` answers `GET /health` on a
//! loopback address of its own, and `iasion health` asks it. Healthy means the
//! database answers; missing Access keys are not a reason to restart.

use std::time::Duration;

use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use sqlx::PgPool;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::config;

pub fn router(pool: PgPool) -> Router {
    Router::new().route("/health", get(health)).with_state(pool)
}

async fn health(State(pool): State<PgPool>) -> (StatusCode, &'static str) {
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => (StatusCode::OK, "ok\n"),
        Err(error) => {
            tracing::warn!(%error, "health check: the database does not answer");
            (StatusCode::SERVICE_UNAVAILABLE, "database unavailable\n")
        }
    }
}

/// Exits 0 if the local instance answers its health check with 200.
pub async fn check() -> Result<(), String> {
    let address = config::health_addr()?;
    let ask = async {
        let mut stream = TcpStream::connect(address).await?;
        stream
            .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await?;
        let mut response = Vec::new();
        stream.read_to_end(&mut response).await?;
        Ok::<_, std::io::Error>(response)
    };
    let response = timeout(Duration::from_secs(5), ask)
        .await
        .map_err(|_| "health: no answer within 5 seconds".to_string())?
        .map_err(|error| format!("health: {error}"))?;
    let status_line = response
        .split(|byte| *byte == b'\n')
        .next()
        .unwrap_or_default();
    if status_line.starts_with(b"HTTP/1.1 200") {
        println!("health: ok");
        Ok(())
    } else {
        Err(format!(
            "health: {}",
            String::from_utf8_lossy(status_line).trim()
        ))
    }
}
