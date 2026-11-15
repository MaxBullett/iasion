# 0012: Transfers between members appear on a separate line in member views

- Status: accepted
- Date: 2026-11-15
- Resolves: OQ-018
- Refined by: ADR-0018, ADR-0019

## Context

Member views attribute positions and flows to members through ownership shares. A
transfer between household accounts with different ownership (e.g. Alex moves
EUR 1,000 from Alex's account into the joint 50/50 account) is invisible in the
household view but shifts attributed wealth between members: Alex -500, Sam +500.

Example (reference household, October): Alex opens at 2,000, earns 3,000, pays half of
600 groceries from the joint account and transfers 1,000 into it; Alex closes at 4,200.
Sam opens at 2,000, earns 2,500, pays the other half of the groceries; Sam closes at
4,700.

## Decision

1. Member views show three kinds of movement: **in**, **out** and **between members**.
2. A transfer between household accounts contributes to a member's between-members
   line the net change in that member's attributed positions it causes (destination
   share times amount minus source share times amount). It never counts as in or out.
3. Each member view reconciles: opening + in - out + between members = closing.
4. Across the household, between-members lines sum to zero, and members' in and out
   sum to the household's in and out.
5. A transfer between two accounts with identical ownership contributes zero and need
   not be listed.

## Alternatives considered

- **Exclude transfers from member views:** member figures stop reconciling (in the
  example, Alex's implied closing is 4,700 against an actual 4,200).
- **Treat as in and out:** member figures reconcile, but members' out no longer sums
  to the household's out (1,100 against 600), and internal moves are presented as
  income and spending.

## Consequences

- Two cheap invariants for property tests: per-member reconciliation, and
  between-members lines summing to zero across the household.
- The between-members line shows who funds shared costs, which the in/out view alone
  cannot.
- Ownership shares become part of every member-view computation; changes to shares
  over time (later) will need the same treatment.
