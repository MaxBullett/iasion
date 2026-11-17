# 0025: Parties, accounts, ownership shares and purposes are corrected as versions

- Status: accepted
- Date: 2026-11-15
- Refines: ADR-0010, ADR-0015
- Refined by: ADR-0029
- Resolves: OQ-023, except how purpose names are compared and how something in use is
  retired

## Context

S-008 corrects transactions and opening balances. ADR-0010 names transactions, opening
balances, checkpoints and dimension assignments as versioned facts. Parties, accounts,
ownership shares and purposes were not mentioned, and no story changed them except
renaming a purpose, although a wrong name or share is as routine as a wrong amount
(`docs/event-storming/r1.md`, H-04). A corrected share changes every member view.

## Decision

1. **Corrected as versions, in R1.** Parties, accounts with their ownership shares,
   and purposes have an identity and versions with recorded time, like other facts
   (ADR-0010).
2. **Names can be corrected freely.**
3. **A corrected ownership share applies retroactively:** "it was always 60/40". Every
   member view uses the corrected shares for all dates. "As known on" shows the
   earlier view.
4. **Nothing has an effective date in R1.** Ownership shares that change over time are
   not R1 (OQ-041).
5. **Rejected:** changing a party's kind to or from system, and moving a party across
   the boundary, i.e. turning a member into a counterparty or the reverse.
6. **Voiding** a party, an account or a purpose is possible only while no current fact
   refers to it (ADR-0015, decision 5).
7. **A party's implicit account is part of the party.** It is voided with it and does
   not count as a fact that refers to it.

## Alternatives considered

- **Edit in place:** no history, and "as known on" would show today's names and shares
  for the past.
- **Shares with an effective date now:** attribution (ADR-0018) would have to pick the
  share valid on each flow's date. No R1 story needs it.
- **Void with a cascade:** voiding an account would silently void its transactions.
- **Free change of party kind:** system parties must stay created by the system only
  (ADR-0011).

## Consequences

- A corrected share changes the allocation of every flow on that account (ADR-0019),
  and with it the member figures of past periods. The invariants hold before and
  after.
- "As known on" covers names, ownership shares and purposes as well as flows. This
  answers that part of OQ-031.
- A correction of owners or shares must keep the shares summing to 1 (ADR-0019) and the
  owners on one side of the boundary (ADR-0023).
- What refers to what: a party is referred to by an account it owns or is the
  institution of, other than its implicit account, and by flows on its accounts; an
  account by its opening balance, flows and checkpoints; a purpose by current
  assignments.
- These things have versions and recorded time but no valid time of their own in R1:
  a version says how it always was.
- Still open in OQ-023: how purpose names are compared, and how an account, a member or
  a purpose that is in use is retired.
- New story S-015. `docs/releases/r1.md` lists it under corrections.
