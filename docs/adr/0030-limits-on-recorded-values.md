# 0030: Limits on recorded values: no future dates, no zero flows, one checkpoint per date

- Status: accepted
- Date: 2026-11-17
- Refines: ADR-0014
- Resolves: OQ-028

## Context

No spec stated a rule for facts dated in the future, flows of zero, negative positions,
or two checkpoints for one account and date, including one on the date of the opening
balance (`docs/event-storming/r1.md`, H-11). Domain code never reads the clock
(`docs/conventions.md`), so a rule about the future needs "today" as an input, and
"today" depends on a time zone.

## Decision

1. **The household has a time zone,** an IANA time zone name. It is set at first setup,
   prefilled from the browser. **Today** is the current civil date in that time zone.
   It is passed to domain code as an input.
2. **No fact is dated after today.** A flow, an opening balance or a checkpoint dated
   after today is rejected with a typed error, and so is a correction that would date
   one after today. For an aggregate flow the last day of its interval counts. Views
   accept any as-of date and any period.
3. **No flow of zero.** A flow whose amount is zero is rejected, except in the
   transaction of an opening balance: an opening balance of 0.00 has two flows of
   zero. A checkpoint of 0.00 is allowed.
4. **Negative positions are allowed** on every account, cash wallets included, and
   checkpoints may be negative. Rejecting them would make the order of entry matter;
   a checkpoint shows the error instead.
5. **One checkpoint per account and date.** A second non-void checkpoint for the same
   account and date is rejected; the first is corrected instead. A checkpoint on the
   date of the account's opening balance is rejected: the opening balance is that
   date's checkpoint (ADR-0011). Both apply to corrections of a checkpoint's date.

## Alternatives considered

- **Allow future dates and mark such facts:** planned flows are projections, which
  have their own concept (generators, later). A fact in the future mixes the two.
- **Today in UTC, or in the server's time zone:** around midnight a member would be
  refused today's date, or allowed tomorrow's.
- **Zero flows as no-ops:** they add nothing to any figure and only make the record
  harder to read.
- **Reject negative positions, or only for cash wallets:** recording a transfer before
  the spending it funds would fail until both exist.
- **Several checkpoints per date, latest wins:** two observed balances for the same end
  of day cannot both be true, and "latest" would mean latest recorded.

## Consequences

- The time zone is household data, versioned and correctable like other household
  data (ADR-0025). Correcting it changes what today is from then on; it never changes
  a recorded date.
- The application layer computes today from the clock and the household's time zone;
  domain code receives it, like the as-of date.
- A cash wallet with a negative position is a hint, not an error. A checkpoint of the
  wallet shows the difference.
- S-001, S-005, S-006 and S-011 gain criteria. The glossary defines today.
- ADR-0015 already rejects moving an opening balance onto the date of a checkpoint;
  decision 5 is the same rule seen from the checkpoint.
