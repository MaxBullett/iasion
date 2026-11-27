//! The one migration sequence for all contexts (ADR-0033, ADR-0034).
//!
//! Files are `migrations/<timestamp>_<context>_<name>.sql`; each touches only its
//! context's schema. They are forward only, never edited once applied, never change
//! what a stored version says, and keep the previous release working (ADR-0033,
//! ADR-0040). `cargo xtask migrations` checks the names and schemas.

/// All migrations, embedded in the binary, with sqlx's defaults. Tests use it.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// The migrations as `iasion migrate` runs them, with the owner role. Migrations in the
/// database that this binary does not embed are accepted: after a rollback, the
/// previous release runs on the schema the newer one migrated (ADR-0040, decision 10),
/// and its migrate step must not stop it from starting.
pub fn release() -> sqlx::migrate::Migrator {
    let mut migrator = sqlx::migrate!("./migrations");
    migrator.set_ignore_missing(true);
    migrator
}
