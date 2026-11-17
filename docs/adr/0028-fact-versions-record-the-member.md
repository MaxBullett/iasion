# 0028: Every fact version records the member who recorded it; sign-ins are not facts

- Status: accepted
- Date: 2026-11-16
- Refines: ADR-0010
- Resolves: OQ-021 (remaining points)

## Context

A request is served in a member's context (S-014, ADR-0024). Apart from the provenance
of purpose assignments (S-010), nothing said what that member is used for, whether a
fact records who recorded it, or whether sign-ins and denied requests are kept
(`docs/event-storming/r1.md`, H-02).

## Decision

1. **Every version of a fact records the member who recorded it,** next to its
   recorded time (ADR-0010).
2. **Sign-ins and denied requests are not domain facts.** They are not stored in the
   database. They appear in operational logs only.

## Alternatives considered

- **Record the member on assignments only:** a correction of an amount or an ownership
  share would not say who made it, in an app whose point is that the past stays
  explainable.
- **Record the login identity instead of the member:** the member is the stable
  thing; login identities come and go (ADR-0024, ADR-0026).
- **An access log in the database:** it is operations, not household finance, and it
  would put every request's email into the data that views and exports read.

## Consequences

- It covers ledger facts (transactions, opening balances, checkpoints, dimension
  assignments) and the versions of parties, accounts and purposes (ADR-0025).
- "Assigned by me" in the provenance of a manual purpose assignment (S-010) is this
  rule applied to an assignment.
- What a command creates on the side, such as the opening party or an implicit
  account, is recorded by the member whose command created it. The versions created by
  first setup are recorded by the first member.
- The member is recorded, not the login identity that was used.
- What the operational logs contain and how long they are kept belongs to the
  authentication enabler and the deployment docs, not to the domain model.
- S-014 gains two criteria.
