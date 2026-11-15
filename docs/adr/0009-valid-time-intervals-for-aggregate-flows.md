# 0009: Valid-time intervals for aggregate flows

- Status: accepted
- Date: 2026-11-15
- Resolves: OQ-017
- Refined by: ADR-0014

## Context

R1 is manual entry, and manual aggregate flows keep capture cheap (e.g. "groceries,
October 2026, EUR 640.00" from the joint account, S-006). Until now a flow happened at
a single valid date.

Booking an aggregate on one date (e.g. the period end) misstates every derived position
inside the period: the account looks EUR 640.00 richer all month, and a checkpoint on
the 15th shows a difference that does not exist.

In R2, imported detail for an account and period must replace the aggregate for the
same account and period, without double counting. That requires knowing which period
an aggregate stands for, which a single date cannot express.

## Decision

1. **Valid time is a date or an interval.** A transaction's valid time is either a
   civil date or a closed interval of civil dates. All flows of a transaction share
   its valid time. Ordinary flows use a date; aggregate flows use an interval.
2. **No proration.** An aggregate flow is never spread across its interval (e.g. evenly
   per day). Spreading would invent precision the data does not have.
3. **Derived positions inside an interval are approximate.** For an as-of date strictly
   inside an aggregate flow's interval, the derived position of each affected account
   is marked approximate. The aggregate is not included in it (it is booked in full at
   the end of the interval), and the view must say so.
4. **Period views:**
   - If the view period fully contains the aggregate's interval, the aggregate counts
     in full, and the figure is exact as far as the aggregate is concerned.
   - If the view period partly overlaps the interval, the aggregate is excluded from
     the figure and the figure is flagged partial, naming the aggregate.
   - If there is no overlap, the aggregate does not affect the figure.
5. **Checkpoints inside an interval** compare against the approximate derived position
   and show the difference as approximate as well.

## Alternatives considered

- **Single date (period end), period kept as metadata:** no kernel change, but the
  kernel would not know the period, so derived positions inside it are silently wrong
  and R2 replacement depends on a field the kernel ignores.
- **Interval with proration:** smooth charts and no approximate flags, but false
  precision, which contradicts the rigour principle and ADR-0007 (missing data is
  never zero, nor invented).

## Consequences

- The Flow and Transaction primitive changes: valid time becomes a date or an interval.
- Every computation over time must handle intervals: derived positions (approximate
  inside an interval), period views (exact, partial, unaffected), checkpoints.
- Views need an "approximate" and a "partial" state in addition to exact and unknown.
  The frontend conventions on exposing incomplete figures in text apply.
- R2 can define replacement precisely: detail imported for account A and interval I
  supersedes aggregates on A whose interval lies within I.
- Statistical generators (later) must treat aggregate flows as interval observations,
  not daily ones.
- Persistence must store valid time as either a date or a date range. In PostgreSQL a
  `daterange` covers both (a single date is a one-day range), but the domain type
  keeps the distinction explicit.
