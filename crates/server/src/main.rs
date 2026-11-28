//! The Iasion binary: configuration and the `serve`, `migrate` and `health`
//! subcommands (ADR-0034, ADR-0040).

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
            --pending: only tell; exit 0 if any is pending, 1 if none, 2 on an error
  health    exit 0 if the local instance is healthy (IASION_HEALTH_ADDR)";

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args == ["migrate", "--pending"] {
        return match database::pending().await {
            Ok(0) => {
                println!("migrate: no migration pending");
                ExitCode::from(1)
            }
            Ok(pending) => {
                println!("migrate: {pending} migration(s) pending");
                ExitCode::SUCCESS
            }
            Err(message) => {
                eprintln!("{message}");
                ExitCode::from(2)
            }
        };
    }
    let command = args.first();
    let result = match command.map(String::as_str) {
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
