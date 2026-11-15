# 0023: In R1 a household account is owned by members only, and the household is not a party

- Status: accepted
- Date: 2026-11-15
- Refines: ADR-0013
- Resolves: OQ-022, except whether an institution must be a party of kind institution

## Context

"Household account" carries the definitions of transfer, net worth and in and out
(glossary, ADR-0013, S-007, S-012, S-013), but was never defined. Since ADR-0013 every
party can own accounts, so an account could have owners on both sides of the boundary.
`docs/primitives.md` also listed "the household itself" as a party, and it was open
whether the boundary is something a member edits (`docs/event-storming/r1.md`, H-03,
H-05).

## Decision

1. **A household account is an account all of whose owners are members.**
2. **No mixed ownership in R1.** The owners of an account are either all members or
   all parties outside the boundary. Registering or correcting an account so that a
   member and a party outside the boundary both own it is rejected with a typed error.
   Mixed ownership is parked for the consolidation release (OQ-039).
3. **The household is not a party.** It owns nothing.
4. **In R1 the boundary is the set of members.** It is not edited on its own.

## Alternatives considered

- **Allow mixed ownership and count only the members' share:** net worth, in and out,
  transfers and attribution would each need a second case for accounts that are partly
  inside.
- **The household as a party that owns joint accounts:** ownership shares already
  express joint ownership, and attribution to members would need another rule for
  what the household holds.
- **An editable boundary:** no R1 story needs it. It belongs with consolidation of
  entities that a member owns.

## Consequences

- Net worth is the sum of the positions of household accounts, with no partial
  accounts. Attribution (ADR-0018, ADR-0019) never meets a share held by a non-member.
- ADR-0013 still holds: every party can own accounts. A party outside the boundary
  owns its implicit account and any account registered for it.
- `docs/primitives.md` no longer lists the household as a party. The glossary defines
  household account.
- S-003 gains a criterion that rejects mixed ownership.
- A member cannot leave the boundary and a counterparty cannot enter it in R1
  (ADR-0025).
- Still open in OQ-022: whether an account's institution must be a party of kind
  institution.
