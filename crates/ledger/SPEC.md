# Ledger context

Part of the context map in [`architecture.md`](../../architecture.md#bounded-contexts)
([ADR-0032](../../docs/adr/0032-r1-bounded-contexts.md)). Downstream of Household,
upstream of Classification and Views. Persistence: schema `ledger`
([ADR-0033](../../docs/adr/0033-persistence-postgresql-versioned-facts.md)).

## Holds

Accounts with owners, ownership shares, institution and implicit accounts; opening
balances; transactions and flows; checkpoints. Facts, and the figures derived from them
alone: derived positions, which flows count in in and out, and the allocation of flows
to members.

## Aggregates and invariants

- **Account:** ownership shares are exact fractions summing to 1 (ADR-0019); owners on
  one side of the boundary (ADR-0023); an institution, if any, of kind institution
  (ADR-0027); at most one non-void opening balance (ADR-0015); one checkpoint per date
  (ADR-0030).
- **Opening balance:** one fact whose checkpoint and transaction are written together
  (ADR-0011, ADR-0015).
- **Transaction:** flows net to zero per unit (ADR-0013); flows dated after the account's
  opening balance and not after today (ADR-0014, ADR-0015, ADR-0030).
- **Checkpoint:** an end-of-day balance (ADR-0014).

## Implements

Household's port "does the Ledger refer to this party?" (ADR-0025, ADR-0027).

## Stories

S-003 to S-008, S-011, S-015 (accounts), S-016.
