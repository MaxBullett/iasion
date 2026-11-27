//! Connections, migrations and the installation's status (ADR-0033, ADR-0040).

use std::collections::HashSet;

use iasion_app::{BoxFuture, StatusError, SystemStatus};
use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use crate::config;

/// Applies pending migrations as the owner role, as a step of its own before a new
/// version starts (ADR-0033, decision 19).
pub async fn migrate() -> Result<(), String> {
    let options = connect_options("IASION_MIGRATE_DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|error| format!("migrate: cannot connect to the database: {error}"))?;
    iasion_migrations::release()
        .run(&pool)
        .await
        .map_err(|error| format!("migrate: {error}"))?;
    println!("migrate: the database is up to date");
    Ok(())
}

/// The application role's pool (ADR-0033, decision 7).
pub async fn connect_app() -> Result<PgPool, String> {
    let options = connect_options("IASION_DATABASE_URL")?;
    PgPoolOptions::new()
        .max_connections(8)
        .connect_with(options)
        .await
        .map_err(|error| format!("cannot connect to the database: {error}"))
}

/// Refuses to start while a migration this binary embeds is not applied (ADR-0040,
/// decision 9). Migrations newer than the binary are fine: after a rollback, the
/// previous release runs on the migrated schema (ADR-0040, decision 10).
pub async fn ensure_migrated(pool: &PgPool) -> Result<(), String> {
    // Before the first `iasion migrate`, sqlx's table does not exist yet.
    let recorded: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('public._sqlx_migrations')::text")
            .fetch_one(pool)
            .await
            .map_err(|error| format!("cannot read the applied migrations: {error}"))?;
    let applied: Vec<i64> = match recorded {
        None => Vec::new(),
        Some(_) => sqlx::query_scalar("SELECT version FROM public._sqlx_migrations WHERE success")
            .fetch_all(pool)
            .await
            .map_err(|error| format!("cannot read the applied migrations: {error}"))?,
    };
    let applied: HashSet<i64> = applied.into_iter().collect();
    let pending = iasion_migrations::MIGRATOR
        .iter()
        .filter(|migration| !applied.contains(&migration.version))
        .count();
    if pending > 0 {
        return Err(format!(
            "{pending} migration(s) pending: run `iasion migrate` first (ADR-0033, decision 19)"
        ));
    }
    Ok(())
}

/// Connection options from the URL in the environment variable `name`. Sessions use
/// UTC (ADR-0033, decision 10). Errors never contain the URL, which holds a password.
fn connect_options(name: &str) -> Result<PgConnectOptions, String> {
    let url = config::required(name)?;
    let options: PgConnectOptions = url
        .parse()
        .map_err(|_| format!("{name} is not a valid PostgreSQL URL"))?;
    Ok(options.options([("TimeZone", "UTC")]))
}

/// The installation's status, read with the application role.
pub struct DatabaseStatus(pub PgPool);

impl SystemStatus for DatabaseStatus {
    fn schema_version(&self) -> BoxFuture<'_, Result<Option<i64>, StatusError>> {
        Box::pin(async {
            sqlx::query_scalar("SELECT max(version) FROM public._sqlx_migrations WHERE success")
                .fetch_one(&self.0)
                .await
                .map_err(|error| {
                    tracing::warn!(%error, "cannot read the schema version");
                    StatusError
                })
        })
    }
}
