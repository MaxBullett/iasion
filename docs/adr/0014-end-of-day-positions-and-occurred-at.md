# 0014: Positions are end-of-day, and a transaction may carry an occurred-at time

- Status: accepted
- Date: 2026-11-15
- Refines: ADR-0009 (decisions 3 and 5), ADR-0011
- Refined by: ADR-0030
- Resolves: OQ-026. Tightens the resolution of OQ-019.

## Context

Valid time is a civil date or an interval of civil dates. Event storming for R1
(`docs/event-storming/r1.md`, H-08) showed that "the position at date D" was not
defined consistently:

- The stories implied that a position at D includes the flows of D, but accepted flows
  dated on the opening date on top of the opening balance. Opening balances and
  checkpoints then sat on different boundaries, although an opening balance is a
  checkpoint (ADR-0011), and an opening checkpoint could stop matching the derived
  position.
- ADR-0009 marks positions approximate for dates "strictly inside" an aggregate flow's
  interval. On the first day the aggregate is excluded as well, and spending may
  already have happened.
- The opening figure of a period view was not defined.

Separately, some sources state a time of day (e.g. broker trade confirmations), and the
order within a day matters for some uses (several buys and sells of one security on one
day, for lot matching). Most sources state only a date.

## Decision

1. **One rule: end of day.** A position at date D includes every flow with a valid date
   on or before D.
2. **Opening balances and checkpoints are end-of-day balances.** An opening balance at
   D is the account's closing balance of D. A checkpoint at D is compared with the
   position at D as defined above.
3. **Flows come after the opening balance.** Apart from the opening transaction
   itself, a flow on an account must be dated strictly after that account's opening
   balance. A flow dated on or before it is rejected. For a valid-time interval, the
   interval must start after it.
4. **Aggregate flows.** For an aggregate flow over the interval from start to end,
   derived positions and checkpoint differences at D are approximate for
   start <= D < end. At the end date and after, the full amount is included. This
   replaces "strictly inside" in ADR-0009, decisions 3 and 5.
5. **Period views.** The opening figure of a period is the end-of-day position on the
   day before the period. The closing figure is the end-of-day position on its last
   day.
6. **This is Iasion's own definition** of "balance at D", not a banking convention. The
   UI asks for the "balance at end of day D".
7. **No invented times.** Valid time stays a civil date or an interval of civil dates.
   A time of day that the source does not provide is never made up.
8. **Occurred-at.** A transaction may carry an optional occurred-at: a UTC instant,
   given only when the source provides one. It orders transactions within the same
   valid date. It never changes which date a transaction belongs to.
9. **Allowed from the start.** No R1 feature uses occurred-at, but the model and
   persistence allow it from the first version.
10. **Intraday prices are series.** Intraday market prices are not ledger facts. A
    series has its own time resolution.

## Alternatives considered

- **Opening balance at start of day, checkpoints at end of day** (what the stories
  implied): a statement's closing balance would have to be entered one day later, and
  the same kind of fact would follow two rules.
- **Valid time as a timestamp:** exact ordering, but most sources state only dates, so
  times would be invented, and time zones would enter every position.
- **Occurred-at replaces the date when present:** two kinds of valid time, and the day
  a transaction belongs to could differ from its source document.
- **A successor to ADR-0009 instead of a refinement:** it would restate all of ADR-0009
  to change two words. ADR-0009 gets a "Refined by" line instead, as ADR-0007 did.

## Consequences

- OQ-019's resolution tightens from "before" to "on or before" the opening balance.
- A statement's closing balance for D is entered as is, dated D. Whatever happened on
  the opening date is part of the opening balance and is not recorded separately until
  backfill (ADR-0011).
- An opening checkpoint always equals the derived position at its date, because no
  other flow on the account can be dated on it.
- The first day of an aggregate flow's interval is approximate. The glossary entry for
  approximate changes accordingly.
- The member reconciliation of ADR-0012 holds for every period that starts after the
  opening balance of every account involved. For a period that starts on or before
  one, the opening figure is unknown; what the view shows then is open (OQ-033).
- Story examples need an opening date before the first recorded month: S-004 and S-005
  move the opening balance to 2026-09-30.
- Occurred-at is part of the transaction, so it is corrected with it (ADR-0010).
- Nothing defines the order of transactions on one date without an occurred-at. R1
  does not need one, since positions are sums. The investment release does (OQ-036).
- Nothing ties occurred-at to the valid date: turning an instant into a date needs a
  time zone. The valid date decides.
- Importers (R2) must choose which date becomes valid time when a statement gives both
  a booking date and a value date (OQ-037).
- The Time sections of `docs/conventions.md` and `docs/primitives.md` state these
  rules.
