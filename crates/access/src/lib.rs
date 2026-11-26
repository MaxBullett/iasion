//! Access context: login identities, who a signed-in request is, and the first setup
//! gate (ADR-0024, ADR-0032). See `SPEC.md`.

#[cfg(feature = "postgres")]
mod pg;
