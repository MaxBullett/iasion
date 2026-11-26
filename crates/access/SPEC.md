# Access context

Part of the context map in [`architecture.md`](../../architecture.md#bounded-contexts)
([ADR-0032](../../docs/adr/0032-r1-bounded-contexts.md)). Downstream of Household.
Persistence: schema `access`
([ADR-0033](../../docs/adr/0033-persistence-postgresql-versioned-facts.md)).

## Holds

Login identities: which member a signed-in request is, and the first setup gate.

## Aggregates and invariants

- **The household's login identities:** a login identity maps to exactly one member; a
  member may have several, or none (ADR-0024); the household keeps at least one
  (ADR-0026); versioned like other household data (ADR-0026); emails stored trimmed and
  lowercased (ADR-0037).

## Implements

Household's port "does a login identity map to this member?" (ADR-0026).

## Stories

S-001 (first setup), S-014.
