//! The Iasion binary: configuration and the `serve` and `migrate` subcommands
//! (ADR-0034).

// Computes today from the clock, so it may read it (ADR-0030, ADR-0034, decision 6).
#![allow(clippy::disallowed_methods)]

use std::env;
use std::process::ExitCode;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

const USAGE: &str = "usage: iasion <command>

commands:
  migrate   apply pending migrations with the owner role (IASION_MIGRATE_DATABASE_URL)";

#[tokio::main]
async fn main() -> ExitCode {
    let command = env::args().nth(1);
    let result = match command.as_deref() {
        Some("migrate") => migrate().await,
        _ => Err(USAGE.to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

/// Applies pending migrations as the owner role, as a step of its own before a new
/// version starts (ADR-0033, decision 19).
async fn migrate() -> Result<(), String> {
    let options = connect_options("IASION_MIGRATE_DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|error| format!("migrate: cannot connect to the database: {error}"))?;
    iasion_migrations::MIGRATOR
        .run(&pool)
        .await
        .map_err(|error| format!("migrate: {error}"))?;
    println!("migrate: the database is up to date");
    Ok(())
}

/// Connection options from the URL in the environment variable `name`. Sessions use
/// UTC (ADR-0033, decision 10). Errors never contain the URL, which holds a password.
fn connect_options(name: &str) -> Result<PgConnectOptions, String> {
    let url = env::var(name).map_err(|_| format!("{name} is not set (see .env.example)"))?;
    let options: PgConnectOptions = url
        .parse()
        .map_err(|_| format!("{name} is not a valid PostgreSQL URL"))?;
    Ok(options.options([("TimeZone", "UTC")]))
}
