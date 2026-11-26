//! Application layer: commands that span contexts as one unit of work, today and the
//! acting member (ADR-0032). With `server`, the only crate that may read the clock
//! (ADR-0034).

// Computes today from the clock, so it may read it (ADR-0030, ADR-0034, decision 6).
#![allow(clippy::disallowed_methods)]
