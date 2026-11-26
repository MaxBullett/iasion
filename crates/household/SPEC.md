# Household context

Part of the context map in [`architecture.md`](../../architecture.md#bounded-contexts)
([ADR-0032](../../docs/adr/0032-r1-bounded-contexts.md)). Upstream of Access, Ledger
and Views. Persistence: schema `household`
([ADR-0033](../../docs/adr/0033-persistence-postgresql-versioned-facts.md)).

## Holds

The household, its members, its boundary and its time zone, and all parties with their
kinds, including the opening party.

## Aggregates and invariants

- **Household:** one household per instance (ADR-0024); exactly one opening party
  (ADR-0011); a time zone, set at first setup (ADR-0030); the boundary is the set of
  members (ADR-0023).
- **Party:** members are parties of kind natural person inside the boundary; a party
  never crosses the boundary and never changes kind to or from system; voided only
  while no current fact refers to it (ADR-0025); names stored trimmed and compared
  ignoring case (ADR-0029).

## Ports it owns

- Does the Ledger refer to this party? Asked before voiding a party or correcting its
  kind away from institution (ADR-0025, ADR-0027).
- Does a login identity map to this member? Asked before voiding a member (ADR-0026).

## Stories

S-001, S-002, S-015 (parties).
