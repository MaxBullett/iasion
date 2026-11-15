# Open questions

Unresolved decisions and unknowns. Agents add entries here instead of assuming.
Format: ID, question, context, status. Resolved entries link to the ADR or spec that
settled them.

## OQ-001: Conditional flows

Do insurance contracts (flows conditional on events) fit the Generator primitive, or do
they need an extension (e.g. event-triggered generators)?

Status: open

## OQ-002: Derivatives

Do option-like instruments (value depends on a series and a payoff function) fit
Position + Generator + Series, or do they need a dedicated valuation abstraction?

Status: open

## OQ-003: Scope of event sourcing

Proposed: event-source contract lifecycles only; the ledger is append-only but not
event-sourced; projections are derived and cached. Needs an ADR.

Update: [ADR-0010](adr/0010-corrections-as-versions.md) stores ledger facts as
versioned facts with recorded time, which covers corrections and "as known on" queries
without full event sourcing. What remains open is whether contract lifecycles go
further.

Status: proposed

## OQ-004: Plugin mechanism

Plugins are compiled-in crates for now (ADR-0005). Is a runtime boundary (e.g. WASM components)
needed later, and what would trigger that decision?

Status: open

## OQ-005: Calibration tooling

Estimating models for the scenario engine (e.g. VAR, GARCH) has weak Rust library
support. Proposed: separate calibration (offline, any tool) from simulation (core),
with calibration producing versioned parameter files. Needs an ADR.

Status: proposed

## OQ-006: Persistence

Database choice not decided.

Status: open

## OQ-007: Contributor licensing

Decide on a CLA or DCO before accepting the first external contribution, to preserve
relicensing options under AGPL-3.0.

Status: open

## OQ-008: Task IDs in the story map

Activities use `A#` and stories use `S-###`. Tasks have no ID scheme yet. Do tasks need
IDs (e.g. for traceability from stories), and in what format?

Status: resolved. Format `A<activity>.<n>`, never reused or renumbered; defined in
`docs/story-map.md`.

## OQ-009: Recording an opening balance, and backfilling before it

Is an opening balance recorded as flows (which, per ADR-0006, need a counterparty: the
suspense party, a dedicated opening party, something else?) or as a distinct kind of
fact that states positions directly? When history before the opening balance is
backfilled later (ADR-0007, principle 1), does the opening balance move to the new
start, become a checkpoint, or get corrected by a new fact?

Status: resolved by [ADR-0011](adr/0011-opening-balance-as-transaction.md) (a
checkpoint taken as given plus a transaction against the household's opening party;
backfill is a correction of that transaction per ADR-0010).

## OQ-010: Booking differences at checkpoints

ADR-0007 books unexplained differences at checkpoints against the suspense party.
Open details:

- Is the suspense flow booked automatically, or proposed and confirmed by the user?
- Which valid time does it get: the checkpoint date, or spread over the interval since
  the previous checkpoint?
- When the difference is explained later (the missing flows are recorded), how is the
  suspense flow reversed, given facts are append-only?
- For accounts holding non-currency units (e.g. securities), is a checkpoint on
  quantities, on value, or both? How does it relate to a valuation?
- Must a checkpoint reference a document, or can the user enter one directly (e.g. a
  balance read off a banking app)?
- What does a difference inside an interval marked complete mean: an error to
  resolve, or a signal that downgrades the interval to partial?

Status: open

## OQ-011: Scope and kind of the suspense party

One suspense party per household, per account, or per member? Is it inside or outside
the boundary? Which party kind is it (aggregate, or a new kind)? Are flows against it
excluded from income and expense views, or shown as a separate line? Are they excluded
when fitting statistical generators?

Input: [ADR-0011](adr/0011-opening-balance-as-transaction.md) introduces the opening
party (one per household, outside the boundary, kind `system`, excluded from in and
out). The suspense party is likely to follow the same pattern.

Status: open

## OQ-012: How coverage is established

Is coverage declared by the user, derived from attached documents (e.g. statement
periods), or both? Does a coverage assignment carry provenance like a dimension? What
distinguishes partial from unknown? Is coverage tracked per account or per unit within
an account?

Status: open

## OQ-013: Classification coverage

ADR-0007 fits statistical generators only on periods with "sufficient classification
coverage" (glossary: classification coverage). This is a different thing from account
coverage (glossary: coverage). Open:

- How it is measured: share of flows by count or by amount? Per dimension?
- Do unconfirmed suggestions count, or only confirmed assignments?
- What threshold counts as sufficient, and where does it live (e.g. in the
  assumption set)?
- Does fitting also require complete coverage of the period? A fully classified
  period with partial coverage would pass a classification test but still be biased.

Status: open

## OQ-014: Representing parameter uncertainty

Statistical generators carry parameter uncertainty into projections, and all
randomness comes from the scenario engine. How are parameter draws represented (e.g.
one draw per scenario path), and are fitted parameter distributions part of the
versioned assumption set?

Status: open

## OQ-015: Non-random missingness

ADR-0007 forbids assuming missingness is random when fitting statistical generators.
Which approach is used instead (e.g. modeling the missingness mechanism, excluding
affected periods, sensitivity bounds, widening uncertainty)? Needs a spec, ideally
with a cited statistical reference.

Status: open

## OQ-016: State of contracts registered mid-life

Contract state is derived from terms plus lifecycle events (`docs/conventions.md`).
Present-first onboarding (ADR-0007) registers open contracts mid-life, usually without
their past lifecycle events (e.g. earlier prepayments or rate resets on a loan). How is
a contract's state at registration captured: an opening state at a date (analogous to
an opening balance), mandatory backfill of past events, or something else? A gap here
would otherwise show up as a checkpoint difference (e.g. against the outstanding
principal on a loan statement) and end up with the suspense party. Related: OQ-003.

Status: open

## OQ-017: Valid time of aggregate flows

R1 allows manual aggregate flows (e.g. "groceries, October 2026, EUR 640.00", S-006).
A flow currently happens at a point in time. Does an aggregate flow get a single valid
date (e.g. the last day of the period), or a valid-time interval? If an interval, how
do derived positions inside the interval and period views that cut through it treat
it (e.g. the as-of balance on the 15th)? Related: R2, where imported detail for a
period must replace an aggregate for the same account and period without double
counting.

Status: resolved by [ADR-0009](adr/0009-valid-time-intervals-for-aggregate-flows.md)
(valid-time interval, no proration).

## OQ-018: Member view of transfers between members

In the household view, transfers between household accounts are excluded from in and
out (S-007, S-013). In a member view, a transfer from Alex's account to Sam's account
is an outflow for one member and an inflow for the other. Is it shown, excluded, or
shown as a separate "between members" line? The same applies to transfers into or out
of joint accounts, where ownership shares differ between the two accounts.

Status: resolved by [ADR-0012](adr/0012-between-members-line.md) (separate "between
members" line, never in or out).

## OQ-019: Flows before the opening balance

R1 has no backfill (ADR-0007, principle 1, is later). Is a flow dated before an
account's opening balance rejected in R1, or accepted and ignored by derived
positions? Proposed for R1: rejected, with a typed error.

Status: resolved. Rejected with a typed error in R1 (S-005); backfill later is a
correction of the opening balance ([ADR-0011](adr/0011-opening-balance-as-transaction.md)).

Update: [ADR-0014](adr/0014-end-of-day-positions-and-occurred-at.md)
makes an opening balance the end-of-day balance of its date, so a flow dated on or
before it is rejected, not only one dated before it.

## OQ-020: Correcting a flow

Facts are append-only and corrections are new facts (`docs/conventions.md`). For a
wrongly entered flow (S-008), is the correction a superseding fact that replaces the
original in all views, or a reversing flow plus a new flow? The first keeps views
clean and maps naturally onto bitemporal queries; the second is the classic
bookkeeping approach and keeps every effect visible as a flow. Also: how is voiding
represented, and can an opening balance be corrected the same way?

Status: resolved by [ADR-0010](adr/0010-corrections-as-versions.md) (new version of
the transaction; voiding is a final version; opening balances version the same way).

## OQ-021: The first household, and mapping sign-ins to members

Raised by R1 event storming (H-01, H-02).

S-014 serves a request only if its Access email maps to a member. S-001 creates the
household "given no household exists", when no member exists yet. Open:

- Who may create the household, and how does the first member's email get mapped?
- May an instance hold more than one household? Multi-household hosting is out of scope
  (`goal-and-requirements.md`), but no spec says a second "create household" is
  rejected.
- Where does the email-to-member mapping live, and which command sets it: an attribute
  of the member, a separate mapping, or instance configuration (r1.md: "No user
  management in the app")? When a member is added (S-001), how do they become able to
  sign in?
- What is the member a request is served for used for in R1, beyond the provenance of
  purpose assignments (S-010)? Does every version of a fact record which member
  recorded it?
- Are sign-ins or denied requests recorded?

Related: the authentication enabler in r1.md (ADR to write).

Status: open

## OQ-022: Account owners, household accounts and the boundary

Raised by R1 event storming (H-03, H-05).

Update: [ADR-0013](adr/0013-signed-flows-on-accounts.md) lets every party own accounts and gives each party outside the boundary an implicit account. What
remains:

- "Household account" is used by the glossary (transfer), ADR-0013 and S-007, S-012 and
  S-013, but is not defined. Is it an account all of whose owners are inside the
  boundary? If a member and a non-member share an account, is it a household account,
  and how do net worth and in and out treat the non-member's share?
- May the household itself, which `docs/primitives.md` lists as a party, own an
  account?
- Is the boundary something a member changes, or in R1 simply the set of members?
- Must an account's institution be a party of kind institution?

Status: open

## OQ-023: Correcting parties, accounts, ownership shares and purposes

Raised by R1 event storming (H-04).

S-008 corrects transactions and opening balances. ADR-0010 names transactions, opening
balances, checkpoints and dimension assignments as versioned facts. Parties, accounts,
ownership shares and purposes are not mentioned, and no story changes them except
renaming a purpose (S-009). Open:

- Is correcting a wrongly entered name, party kind, institution or ownership share part
  of R1?
- Are these versioned like facts (ADR-0010)? r1.md requires valid time and recorded
  time on "every fact". Is a previous name kept?
- A corrected ownership share changes every member view retroactively. Is that
  intended, and how does it differ from a change of shares over time (later, ADR-0012)?
- Closing an account, removing a member, retiring a purpose: confirm these are not R1.
- Purpose names are unique (S-009). How are names compared (case, surrounding
  whitespace), and does renaming to an existing name fail the same way?

Status: open

## OQ-024: Representation of ownership shares and rounding of attributed amounts

Raised by R1 event storming (H-06).

Ownership shares sum to 100% and facts use exact decimals. Open:

- How is a share represented: a percentage with fixed precision, a fraction, something
  else? Three equal owners cannot be written as decimals that sum to exactly 100%.
- How are attributed amounts rounded? Half of EUR 54.21 is not a whole cent. Do member
  figures keep sub-cent precision, or is there a rounding rule that allocates the
  remainder?

Whatever the answer, member figures must still sum to the household figures and
between-members lines must sum to zero (S-012, S-013, ADR-0012).

Status: open

## OQ-025: Shape of a flow and its counterparty end

Raised by R1 event storming (H-07).

`docs/primitives.md`: "A flow moves a quantity of a unit from one position to another",
and positions are held in accounts. The stories record flows "from ACME GmbH into the
account" (S-005) and a transaction "between the account and the opening party" (S-004).
Open:

- Is a flow a movement with two ends, or a signed entry on one position, with the
  transaction holding several? With two ends, every flow nets to zero by construction,
  so the netting invariant can never fail and S-008's "a version that does not net to
  zero is rejected" cannot occur. With signed entries, S-007's "the fee is a second
  flow" no longer fits: a withdrawal with a fee is more than two entries.
- What is the counterparty end: the party itself, or a position in an account of that
  party? ADR-0011 speaks of the opening party's "balance per account".
- Must every transaction have at least one end in a household account?
- May a member be the counterparty end directly, e.g. money handed to Sam that enters
  no registered account? Is that in, out or between members?

Status: resolved by [ADR-0013](adr/0013-signed-flows-on-accounts.md): flows are signed
and one-sided and can fail to net; every flow is on an account, with an implicit account
per party outside the boundary; a transaction need not touch a household account; a
member is never a counterparty directly.

## OQ-026: Day boundary of positions, opening balances and checkpoints

Raised by R1 event storming (H-08).

Valid time is a civil date. The stories imply that the derived position at a date
includes the flows of that date (S-005). Open:

- Is an opening balance at date D the balance before or after the flows dated D? S-005
  rejects only flows dated before D, so flows dated D are added on top, which reads as
  "before". A statement's closing balance for D would then have to be entered at D+1.
  And once a flow dated D exists, the derived position at D no longer equals the
  opening checkpoint (S-004, ADR-0011).
- Is a checkpoint at D compared with the position including all flows dated D (S-011
  suggests so)? Then opening balances and other checkpoints use different boundaries,
  although an opening balance is a checkpoint (ADR-0011).
- ADR-0009 marks positions approximate for as-of dates "strictly inside" an aggregate
  flow's interval. On the interval's first day the aggregate is excluded as well, and
  spending may already have happened. Exact or approximate?
- In period views, is a member's opening figure (S-013, ADR-0012) the position at the
  end of the day before the period, or at the start of its first day? The ADR-0012
  example shows an opening of 2,000 for October.
- If the answer follows a banking convention (e.g. statement balances are end-of-day
  balances), it needs a primary source (`AGENTS.md`, rule 2).

Status: resolved by [ADR-0014](adr/0014-end-of-day-positions-and-occurred-at.md):
one rule,
end of day. A position at a date includes every flow dated on or before it; opening
balances and checkpoints are end-of-day balances; flows are dated after the opening
balance; positions are approximate from the first day of an aggregate interval up to,
but not including, its last; a period's opening figure is the position on the day
before it. This is Iasion's own definition, not a banking convention.

## OQ-027: Life of an opening balance, and accounts without one

Raised by R1 event storming (H-09, H-10).

- An opening balance is corrected to a later date than flows or checkpoints already
  recorded on the account. Rejected, or what happens to those facts? (ADR-0011 only
  covers moving it earlier, as backfill.)
- May an opening balance be voided while the account has flows or checkpoints? After
  voiding, may a new one be recorded ("at most one", S-004)?
- Is an opening balance one fact with its own identity (ADR-0010 lists it as a kind of
  fact) or two linked facts, a checkpoint and a transaction (ADR-0011: "a new version
  of the opening transaction and its checkpoint")? Which identity does a correction
  target, and which aggregate enforces "at most one per account"?
- An account without an opening balance: is a flow that touches it rejected or
  accepted? OQ-019 only settled flows dated before an existing opening balance. This
  includes being the destination of a transfer. Is a checkpoint on it accepted?
- An aggregate flow whose interval starts on or before the opening date and ends after
  it: rejected as a whole? ADR-0014 says yes: the interval must start after the
  opening date.
- Implicit accounts have no opening balance and accept flows without one (ADR-0013).
  Do the opening-balance rules apply to accounts registered for a party outside the
  boundary (e.g. a loan account), or to household accounts only?

Status: open

## OQ-028: Limits on recorded values

Raised by R1 event storming (H-11).

No spec states a rule for:

- Valid time after the day of recording: a flow or checkpoint dated in the future, or a
  view with an as-of date in the future. Domain code never reads the clock
  (`docs/conventions.md`), so any rule needs "now" as an input.
- A flow with amount zero, or an opening balance of zero.
- Negative derived positions. An overdraft is plausible; a cash wallet cannot be
  negative in the world, but rejecting it would make the order of entry matter.
- Two checkpoints for the same account and date. This includes a checkpoint on the
  date of the opening balance, whose difference no flow could explain (ADR-0014).

Status: open

## OQ-029: Flow identity across transaction versions, and purpose assignments

Raised by R1 event storming (H-14).

A purpose is assigned to a flow (S-010) and the assignment is a versioned fact of its
own (ADR-0010). The transaction is the versioned unit, and a correction restates the
whole transaction (ADR-0010). Open:

- Does a flow keep a stable identity across versions of its transaction?
- What happens to a flow's assignment when a correction changes its amount, moves it
  to another account, removes it or splits it in two?
- What happens to the assignments of a voided transaction?
- Is there a limit to what a correction may change (accounts, parties, valid time,
  number of flows), or is the fact's ID the only thing that stays?
- Removing a purpose without replacing it: is that voiding the assignment?
- Provenance includes a confidence (glossary). What is it for a manual assignment?

Related: OQ-034 (what an assignment attaches to).

Status: open

## OQ-030: Concurrent corrections

Raised by R1 event storming (H-15).

Two members may correct or void the same fact, or record an opening balance for the
same account, at the same time. Must a correction name the version it replaces and be
rejected when that version is no longer current? Otherwise the last one silently wins.

Status: open

## OQ-031: "As known on" in R1

Raised by R1 event storming (H-16).

S-008 requires viewing "figures as known on a date before the correction was recorded".
r1.md says R1 has no UI for recorded time. Open:

- Is "as known on" available through the API only in R1, or does S-008 need a UI?
- Recorded time is a UTC timestamp (`docs/conventions.md`). "As known on a date" needs
  a rule for which instant of which day is meant, including the time zone.
- Do names, ownership shares and purposes also apply as known on that time (OQ-023)?

Status: open

## OQ-032: Finding recorded transactions

Raised by R1 event storming (H-17).

Correcting a flow (S-008), assigning a purpose (S-010), recording the flows missing at
a checkpoint (S-011) and the R1 done criterion "every checkpoint difference can be
explained" all presuppose that a member can find recorded transactions, e.g. per
account and period. No R1 story provides that. Is a story missing, or is it meant to be
part of an existing one?

Also: no story gives a transaction a free-text note. Intended?

Status: open

## OQ-033: Figure states and other open points in the R1 views

Raised by R1 event storming (H-21, H-22).

- Figures can be exact, approximate, partial or unknown (ADR-0009). How do these
  combine in a total? S-012 only says approximate propagates. What is net worth when
  one account is unknown (as-of date before its opening balance)? The glossary defines
  partial for period figures only.
- S-013: for a period starting before an account's opening balance, "the figures for
  that account are partial". The view is grouped by purpose, not by account. Which
  figures are marked?
- ADR-0012 requires opening + in - out + between members = closing. With ADR-0014 this
  holds for a period that starts after every opening balance. If a period starts on or
  before an account's opening balance, the opening figure is unknown and the opening
  flow is excluded from in (ADR-0011). What does the member view show then?
- S-012 shows a difference for an account's "latest checkpoint". Latest overall, or
  latest at or before the as-of date? What if an earlier checkpoint has a difference
  and a later one has none?
- A refund is a new transaction (ADR-0010). In in and out, is it an inflow under its
  purpose, or does it reduce the outflow?
- The stories say "income" and "spending" (S-005, S-006, ADR-0011) as well as "in" and
  "out" (S-013, ADR-0012). Are these the same, and which terms go into the glossary?

Status: open

## OQ-034: Where a purpose assignment attaches

Raised by R1 event storming (H-24), after ADR-0013.

S-010 assigns a purpose to a flow. With signed, one-sided flows (ADR-0013), open:

- Does a purpose attach to one flow or to the whole transaction?
- If to a flow, on which side: the flow on the household account, or the flow on the
  counterparty's account? Household in and out are read from the flows on
  non-household accounts (ADR-0013). In a split, e.g. joint current account -80.00,
  Corner Grocer +50.00 and +30.00, the flows that differ by purpose are on the
  counterparty's account. In a cash withdrawal with a fee, only the flow on the bank's
  account is out.
- If to the transaction, a split cannot be one transaction, so one statement line
  would no longer be one flow.
- Which flows may carry a purpose? The flows of a transfer and flows on a system
  party's account are excluded from in and out (S-013). Is assigning a purpose to them
  rejected, or accepted and ignored?
- An aggregate flow has a flow on each side like any other transaction. Same answer?

Related: OQ-029.

Status: open

## OQ-035: Member attribution in transactions with flows in both directions

Raised by R1 event storming (H-25), after ADR-0013.

Member views attribute through ownership shares, and each member reconciles: opening +
in - out + between members = closing (ADR-0012). ADR-0012 words its rule for a transfer
with one source, one destination and one amount. A transaction can now have any number
of flows (ADR-0013), and household in and out are the flows on non-household accounts.
Which member is each of them attributed to?

- When all flows on household accounts point the same way, attributing in proportion
  to them suggests itself, but no spec says so. Example: ACME GmbH -3,800.00, Alex's
  current account +3,000.00, joint current account +800.00.
- When they point both ways and the accounts have different owners, the answer is not
  determined. Example: joint current account -202.00 (Alex 50%, Sam 50%), Alex's cash
  wallet +200.00, Example Bank +2.00. Attributed positions change by +99.00 for Alex
  and -101.00 for Sam, and household out is 2.00. If the fee follows the account it
  left, each has out 1.00 and between members is +100.00 and -100.00. If it follows
  the member who got the cash, Alex has out 2.00 and between members is +101.00 and
  -101.00. Both reconcile.
- Deriving movements (ADR-0013) needs the same decomposition: which account did the
  fee come from?
- Does ADR-0012 need a successor?

Status: open

## OQ-036: Order of same-day transactions without an occurred-at

Raised in review of OQ-026. For the investment release.

ADR-0014 lets a transaction carry an optional occurred-at, which orders transactions
within one valid date. When the source gives none, the order of several buys and sells
of one security on the same date is not defined, and lot matching depends on it. Open:

- What decides the order then (e.g. order of recording, order in the source document,
  a sequence stated by the user), or is a method used that does not depend on it?
- How do transactions with and without an occurred-at on the same date order relative
  to each other?
- A lot matching method prescribed by tax law is a domain rule and needs a primary
  source (`AGENTS.md`, rule 2).

Status: open, parked until the investment release

## OQ-037: Booking date and value date in imports

Raised in review of OQ-026 (H-26). For R2.

Bank statements distinguish a booking date and a value date, and statement balances
follow the booking date. This is stated by the domain expert; once it becomes a rule in
a spec it needs a primary source (`AGENTS.md`, rule 2). Open:

- Which of the two dates becomes the valid time of an imported transaction? With the
  value date, positions derived at a date would not match statement balances at that
  date, and checkpoints would show differences around it.
- Is the other date kept, and where?
- Does the answer differ per importer or source?

Related: ADR-0014, OQ-010.

Status: open, parked until R2
