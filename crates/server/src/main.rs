//! The Iasion binary: configuration and the `serve`, `migrate` and `health`
//! subcommands (ADR-0034, ADR-0040).

// Computes today from the clock, so it may read it (ADR-0030, ADR-0034, decision 6).
#![allow(clippy::disallowed_methods)]

mod config;
mod database;
mod health;
mod keys;
mod notify;
mod serve;

use std::env;
use std::process::ExitCode;

const USAGE: &str = "usage: iasion <command>

commands:
  serve     serve the app (settings in .env.example)
  migrate   apply pending migrations with the owner role (IASION_MIGRATE_DATABASE_URL)
  health    exit 0 if the local instance is healthy (IASION_HEALTH_ADDR)";

#[tokio::main]
async fn main() -> ExitCode {
    let command = env::args().nth(1);
    let result = match command.as_deref() {
        Some("serve") => serve::run().await,
        Some("migrate") => database::migrate().await,
        Some("health") => health::check().await,
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
