# 0002: Rust for the backend

- Status: accepted
- Date: 2026-09-25

## Context

The backend holds a rich domain model (nested contract structures), must stay
maintainable for years, will run async jobs against external data sources, and runs
Monte Carlo projections. It is largely written by AI agents.

## Decision

Use Rust for the backend and kernel.

## Alternatives considered

- Python: best numerical and statistical ecosystem, but weak typing for a long-lived
  domain model.
- Kotlin, C#/.NET: strong typing and mature ecosystems; less expressive sum types
  (C#) and heavier runtimes.

## Consequences

- Sum types and exhaustive matching fit contract structures well.
- The compiler gives agents a tight, specific feedback loop.
- Mature async and data libraries are available.
- No stable ABI: plugins are compiled in (see ADR-0005).
- Statistical estimation libraries are weak; see OQ-005 on separating calibration
  from simulation.
