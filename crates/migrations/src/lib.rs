//! The one migration sequence for all contexts (ADR-0033, ADR-0034).
//!
//! Files are `migrations/<timestamp>_<context>_<name>.sql`; each touches only its
//! context's schema. They are forward only, never edited once applied, never change
//! what a stored version says, and keep the previous release working (ADR-0033,
//! ADR-0040). `cargo xtask migrations` checks the names and schemas.

/// All migrations, embedded in the binary. Run by `iasion migrate` with the owner role.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
