# Classification context

Part of the context map in [`architecture.md`](../../architecture.md#bounded-contexts)
([ADR-0032](../../docs/adr/0032-r1-bounded-contexts.md)). Downstream of Ledger,
upstream of Views. Persistence: schema `classification`
([ADR-0033](../../docs/adr/0033-persistence-postgresql-versioned-facts.md)).

## Holds

The purpose vocabulary, and purpose assignments on flows with their provenance.

## Aggregates and invariants

- **Purpose:** names unique, stored trimmed and compared ignoring case; renaming to
  another purpose's name is rejected (ADR-0029); voided only while no current assignment
  refers to it (ADR-0025).
- **Dimension assignment:** attaches only to a flow that can carry a purpose, as Ledger
  says (ADR-0016); follows its flow, and is not current once its flow is not
  (ADR-0016, ADR-0017).

## Stories

S-009, S-010, S-015 (purposes).
