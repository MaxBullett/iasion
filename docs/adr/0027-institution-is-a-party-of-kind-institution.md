# 0027: An account's institution is a party of kind institution

- Status: accepted
- Date: 2026-11-16
- Refines: ADR-0023
- Resolves: OQ-022 (last point)

## Context

An account is optionally held at an institution (`docs/primitives.md`), and parties
have a kind, one of which is institution. Nothing said whether the party named as an
account's institution must be of that kind (`docs/event-storming/r1.md`, H-05).

## Decision

1. **If an account names an institution, it is a party of kind institution.** Any
   other kind is rejected with a typed error.
2. **The institution stays optional.** An account held at nobody, such as a cash
   wallet, has none.

## Alternatives considered

- **Any party may be the institution:** the kind would say nothing, and views that
  group accounts by institution would list employers and private persons.
- **Derive the kind from use** (a party becomes an institution once an account is held
  at it): a party's kind would change as a side effect of registering an account.

## Consequences

- The rule holds for corrections too (ADR-0017, ADR-0025): correcting an account's
  institution to a party of another kind is rejected, and so is correcting a party's
  kind away from institution while an account names it as its institution.
- An account that is held at something which is not an institution, e.g. a loan from
  a private person, has no institution.
- A party of kind institution is outside the boundary and can also be a counterparty,
  e.g. Example Bank for a fee (S-007).
- S-003 gains a criterion.
