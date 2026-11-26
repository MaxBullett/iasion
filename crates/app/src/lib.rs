//! Application layer: commands that span contexts as one unit of work, today and the
//! acting member (ADR-0032). With `server`, the only crate that may read the clock
//! (ADR-0034).

// Computes today from the clock, so it may read it (ADR-0030, ADR-0034, decision 6).
#![allow(clippy::disallowed_methods)]

use std::future::Future;
use std::pin::Pin;

/// A boxed future that can be sent between threads, for ports used behind `dyn`.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The state of the installation, for the walking skeleton's round trip from the API
/// to the database. `server` implements it on the database.
pub trait SystemStatus: Send + Sync {
    /// The version of the latest applied migration, or `None` if none is applied.
    fn schema_version(&self) -> BoxFuture<'_, Result<Option<i64>, StatusError>>;
}

/// The status could not be read. Carries no detail that could hold a secret.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusError;
