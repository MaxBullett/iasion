//! Ledger context: accounts with owners and ownership shares, opening balances,
//! transactions and flows, checkpoints, and the figures derived from them alone
//! (ADR-0032). See `SPEC.md`.

#[cfg(feature = "postgres")]
mod pg;
