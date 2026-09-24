# 0005: Plugins as compiled-in crates, independent of the frontend

- Status: accepted
- Date: 2026-09-25

## Context

Domain-specific functionality (instruments, importers, jurisdictions, strategies) must
be pluggable. Rust has no stable ABI for runtime-loaded libraries.

## Decision

- Plugins are Rust crates implementing kernel traits, registered at build time.
- Plugins do not ship UI. The frontend renders from the kernel's data model and API.

## Alternatives considered

- Runtime plugins via dynamic libraries: no stable ABI, fragile.
- WASM components: clean runtime boundary, but premature complexity now.
- Plugins contributing frontend components: flexible, but couples UI to plugins.

## Consequences

- Adding a plugin means rebuilding. Acceptable for a self-hosted app.
- The kernel's data model must be expressive enough for the UI to present any plugin's
  output.
- A runtime boundary remains possible later (OQ-004).
